/**
 * H.264 video player using WebCodecs API.
 * Receives raw H.264 Annex-B frames via HTTP streaming,
 * decodes them with hardware-accelerated WebCodecs VideoDecoder,
 * and renders to a Canvas.
 */
export class MsePlayer {
  private canvas: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;
  private video: HTMLVideoElement;
  private streamUrl: string;
  private decoder: VideoDecoder | null = null;
  private abortController: AbortController | null = null;
  private frameCount = 0;
  private sps: Uint8Array | null = null;
  private pps: Uint8Array | null = null;
  private codecDescription: Uint8Array | null = null;
  private configured = false;
  private mediaRecorder: MediaRecorder | null = null;
  private recordedChunks: Blob[] = [];
  // Audio
  private audioDecoder: AudioDecoder | null = null;
  private audioContext: AudioContext | null = null;
  private audioAbortController: AbortController | null = null;
  private audioStartTime = 0;
  private audioSampleOffset = 0;
  private audioCodec: string;

  constructor(video: HTMLVideoElement, streamUrl: string, audioCodec = "opus") {
    this.video = video;
    this.streamUrl = streamUrl;
    this.audioCodec = audioCodec;
  }

  async start(): Promise<void> {
    console.log("[WC] Starting WebCodecs player");

    // Create canvas to replace the video element
    this.canvas = document.createElement("canvas");
    this.canvas.className = "h-full rounded-lg shadow-2xl bg-neutral-900";
    this.canvas.style.cssText = "object-fit: contain; max-height: 100%; max-width: 100%;";
    this.video.parentElement?.replaceChild(this.canvas, this.video);
    this.ctx = this.canvas.getContext("2d");

    // Create decoder
    this.decoder = new VideoDecoder({
      output: (frame) => this.onFrame(frame),
      error: (e) => console.error("[WC] Decoder error:", e),
    });

    console.log("[WC] Decoder created, fetching stream...");

    this.abortController = new AbortController();
    this.fetchStream(this.abortController.signal);

    // Start audio
    this.startAudio();
  }

  private onFrame(frame: VideoFrame): void {
    if (!this.canvas || !this.ctx) {
      frame.close();
      return;
    }

    // Resize canvas internal resolution to match video
    if (
      this.canvas.width !== frame.displayWidth ||
      this.canvas.height !== frame.displayHeight
    ) {
      this.canvas.width = frame.displayWidth;
      this.canvas.height = frame.displayHeight;
      console.log("[WC] Canvas resized:", frame.displayWidth, "x", frame.displayHeight);
    }

    this.ctx.drawImage(frame, 0, 0);
    frame.close();

    this.frameCount++;
    if (this.frameCount <= 3 || this.frameCount % 300 === 0) {
      console.log("[WC] Rendered frame", this.frameCount);
    }
  }

  private configureDecoder(sps: Uint8Array, pps: Uint8Array): void {
    if (!this.decoder) return;

    // Build avcC box for codec description
    this.codecDescription = this.buildAvcC(sps, pps);

    const codec = `avc1.${sps[1].toString(16).padStart(2, "0")}${sps[2].toString(16).padStart(2, "0")}${sps[3].toString(16).padStart(2, "0")}`;

    console.log("[WC] Configuring decoder: codec=", codec);

    this.decoder.configure({
      codec,
      optimizeForLatency: true,
      hardwareAcceleration: "prefer-hardware",
      description: this.codecDescription,
    });

    this.configured = true;
    console.log("[WC] Decoder configured");
  }


  private buildAvcC(sps: Uint8Array, pps: Uint8Array): Uint8Array {
    // avcC structure (without box header)
    const size = 6 + 2 + sps.length + 1 + 2 + pps.length;
    const buf = new Uint8Array(size);
    let i = 0;
    buf[i++] = 1; // configurationVersion
    buf[i++] = sps[1]; // AVCProfileIndication
    buf[i++] = sps[2]; // profile_compatibility
    buf[i++] = sps[3]; // AVCLevelIndication
    buf[i++] = 0xff; // lengthSizeMinusOne = 3
    buf[i++] = 0xe1; // numSPS = 1
    buf[i++] = (sps.length >> 8) & 0xff;
    buf[i++] = sps.length & 0xff;
    buf.set(sps, i);
    i += sps.length;
    buf[i++] = 1; // numPPS
    buf[i++] = (pps.length >> 8) & 0xff;
    buf[i++] = pps.length & 0xff;
    buf.set(pps, i);
    return buf;
  }

  private async fetchStream(signal: AbortSignal): Promise<void> {
    try {
      const response = await fetch(this.streamUrl, { signal });
      if (!response.ok || !response.body) {
        console.error("[WC] Fetch failed:", response.status);
        return;
      }
      console.log("[WC] Stream connected");

      const reader = response.body.getReader();
      let buffer = new Uint8Array(0);

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        // Append to buffer
        const newBuf = new Uint8Array(buffer.length + value.length);
        newBuf.set(buffer);
        newBuf.set(value, buffer.length);
        buffer = newBuf;

        // Parse length-prefixed frames
        while (buffer.length >= 4) {
          const frameLen =
            (buffer[0] << 24) |
            (buffer[1] << 16) |
            (buffer[2] << 8) |
            buffer[3];

          if (frameLen <= 0 || frameLen > 10_000_000) {
            console.error("[WC] Invalid frame length:", frameLen);
            buffer = new Uint8Array(0);
            break;
          }

          if (buffer.length < 4 + frameLen) break;

          const frame = buffer.slice(4, 4 + frameLen);
          buffer = buffer.slice(4 + frameLen);

          this.processH264Frame(frame);
        }
      }
    } catch (e) {
      if (e instanceof Error && e.name !== "AbortError") {
        console.error("[WC] Stream error:", e);
      }
    }
  }

  private processH264Frame(data: Uint8Array): void {
    // Parse NAL units from Annex-B data
    const nals = this.splitNals(data);

    let hasVideoNal = false;
    let isKeyframe = false;

    for (const nal of nals) {
      if (nal.length === 0) continue;
      const nalType = nal[0] & 0x1f;

      switch (nalType) {
        case 7: // SPS
          this.sps = nal;
          break;
        case 8: // PPS
          this.pps = nal;
          if (this.sps && !this.configured) {
            this.configureDecoder(this.sps, this.pps);
          }
          break;
        case 5: // IDR
          isKeyframe = true;
          hasVideoNal = true;
          break;
        case 1: // Non-IDR
          hasVideoNal = true;
          break;
      }
    }

    if (!hasVideoNal || !this.configured || !this.decoder) return;
    if (this.decoder.state !== "configured") return;

    // Convert Annex-B to AVC format (length-prefixed NALs) for WebCodecs
    const avcData = this.annexBToAvc(data);

    try {
      this.decoder.decode(
        new EncodedVideoChunk({
          type: isKeyframe ? "key" : "delta",
          timestamp: this.frameCount * 16666, // ~60fps in microseconds
          data: avcData,
        }),
      );
    } catch (e) {
      console.error("[WC] Decode error:", e);
    }
  }

  /** Convert Annex-B (start codes) to AVC (length-prefixed) format */
  private annexBToAvc(data: Uint8Array): Uint8Array {
    const nals = this.splitNals(data);
    // Filter to only video NALs (not SPS/PPS/SEI)
    const videoNals = nals.filter((nal) => {
      const t = nal[0] & 0x1f;
      return t === 1 || t === 5; // non-IDR or IDR
    });

    // Calculate total size
    let totalSize = 0;
    for (const nal of videoNals) {
      totalSize += 4 + nal.length;
    }

    const result = new Uint8Array(totalSize);
    let offset = 0;
    for (const nal of videoNals) {
      // 4-byte big-endian length
      result[offset++] = (nal.length >> 24) & 0xff;
      result[offset++] = (nal.length >> 16) & 0xff;
      result[offset++] = (nal.length >> 8) & 0xff;
      result[offset++] = nal.length & 0xff;
      result.set(nal, offset);
      offset += nal.length;
    }

    return result;
  }

  /** Split Annex-B byte stream into NAL units */
  private splitNals(data: Uint8Array): Uint8Array[] {
    const nals: Uint8Array[] = [];
    let i = 0;

    while (i < data.length) {
      // Find start code (00 00 00 01 or 00 00 01)
      let start: number;
      if (
        i + 4 <= data.length &&
        data[i] === 0 && data[i + 1] === 0 && data[i + 2] === 0 && data[i + 3] === 1
      ) {
        start = i + 4;
      } else if (
        i + 3 <= data.length &&
        data[i] === 0 && data[i + 1] === 0 && data[i + 2] === 1
      ) {
        start = i + 3;
      } else {
        i++;
        continue;
      }

      // Find next start code
      let end = start;
      while (end < data.length) {
        if (
          end + 4 <= data.length &&
          data[end] === 0 && data[end + 1] === 0 && data[end + 2] === 0 && data[end + 3] === 1
        ) break;
        if (
          end + 3 <= data.length &&
          data[end] === 0 && data[end + 1] === 0 && data[end + 2] === 1
        ) break;
        end++;
      }

      if (end > start) {
        nals.push(data.slice(start, end));
      }
      i = end;
    }

    return nals;
  }

  /** Capture current canvas frame as PNG and trigger download */
  async takeScreenshot(): Promise<void> {
    if (!this.canvas) return;

    const blob = await new Promise<Blob | null>((resolve) =>
      this.canvas!.toBlob(resolve, "image/webp", 0.75),
    );
    if (!blob) return;

    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `screenshot_${new Date().toISOString().replace(/[:.]/g, "-")}.webp`;
    a.click();
    URL.revokeObjectURL(url);
  }

  /** Start recording canvas as WebM video */
  startRecording(): void {
    if (!this.canvas || this.mediaRecorder) return;

    const stream = this.canvas.captureStream(30);
    const mimeType = MediaRecorder.isTypeSupported("video/webm;codecs=vp9")
      ? "video/webm;codecs=vp9"
      : "video/webm";

    this.recordedChunks = [];
    this.mediaRecorder = new MediaRecorder(stream, {
      mimeType,
      videoBitsPerSecond: 8_000_000,
    });

    this.mediaRecorder.ondataavailable = (e) => {
      if (e.data.size > 0) {
        this.recordedChunks.push(e.data);
      }
    };

    this.mediaRecorder.start(1000); // chunk every second
  }

  /** Stop recording and trigger download */
  stopRecording(): void {
    if (!this.mediaRecorder || this.mediaRecorder.state === "inactive") return;

    this.mediaRecorder.onstop = () => {
      const blob = new Blob(this.recordedChunks, { type: "video/webm" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `recording_${new Date().toISOString().replace(/[:.]/g, "-")}.webm`;
      a.click();
      URL.revokeObjectURL(url);
      this.recordedChunks = [];
    };

    this.mediaRecorder.stop();
    this.mediaRecorder = null;
  }

  private async startAudio(): Promise<void> {
    const audioUrl = this.streamUrl.replace("/stream", "/audio");

    // Audio config based on codec (Opus for Android, AAC-ELD for iOS)
    const isAac = this.audioCodec.startsWith("aac");
    const sampleRate = isAac ? 44100 : 48000;
    const codec = isAac ? "mp4a.40.2" : "opus";

    try {
      this.audioContext = new AudioContext({ sampleRate });

      this.audioDecoder = new AudioDecoder({
        output: (audioData) => this.onAudioData(audioData),
        error: (e) => console.error("[Audio] Decoder error:", e),
      });

      this.audioDecoder.configure({
        codec,
        sampleRate,
        numberOfChannels: 2,
      });

      this.audioAbortController = new AbortController();
      this.fetchAudioStream(this.audioAbortController.signal, audioUrl);
    } catch (e) {
      console.warn("[Audio] Failed to start audio:", e);
    }
  }

  private onAudioData(audioData: AudioData): void {
    if (!this.audioContext) {
      audioData.close();
      return;
    }

    const numFrames = audioData.numberOfFrames;
    const numChannels = audioData.numberOfChannels;
    const sampleRate = audioData.sampleRate;
    const buffer = this.audioContext.createBuffer(numChannels, numFrames, sampleRate);

    for (let ch = 0; ch < numChannels; ch++) {
      const channelData = new Float32Array(numFrames);
      audioData.copyTo(channelData, { planeIndex: ch, format: "f32-planar" });
      buffer.copyToChannel(channelData, ch);
    }
    audioData.close();

    const source = this.audioContext.createBufferSource();
    source.buffer = buffer;
    source.connect(this.audioContext.destination);

    const now = this.audioContext.currentTime;
    const frameDuration = numFrames / sampleRate;

    if (this.audioStartTime === 0) {
      // Add 150ms buffer to absorb network jitter
      this.audioStartTime = now + 0.15;
      this.audioSampleOffset = 0;
    }

    let playTime = this.audioStartTime + this.audioSampleOffset;

    // If we've fallen too far behind, reset the schedule
    if (playTime < now - 0.5) {
      this.audioStartTime = now + 0.05;
      this.audioSampleOffset = 0;
      playTime = this.audioStartTime;
    }

    source.start(Math.max(playTime, now));
    this.audioSampleOffset += frameDuration;
  }

  private async fetchAudioStream(
    signal: AbortSignal,
    url: string,
  ): Promise<void> {
    try {
      const response = await fetch(url, { signal });
      if (!response.ok || !response.body) return;

      const reader = response.body.getReader();
      let buffer = new Uint8Array(0);
      let timestamp = 0;

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const newBuf = new Uint8Array(buffer.length + value.length);
        newBuf.set(buffer);
        newBuf.set(value, buffer.length);
        buffer = newBuf;

        while (buffer.length >= 4) {
          const frameLen =
            (buffer[0] << 24) |
            (buffer[1] << 16) |
            (buffer[2] << 8) |
            buffer[3];

          if (frameLen <= 0 || frameLen > 100_000) {
            buffer = new Uint8Array(0);
            break;
          }
          if (buffer.length < 4 + frameLen) break;

          const frame = buffer.slice(4, 4 + frameLen);
          buffer = buffer.slice(4 + frameLen);

          if (this.audioDecoder && this.audioDecoder.state === "configured") {
            try {
              this.audioDecoder.decode(
                new EncodedAudioChunk({
                  type: "key",
                  timestamp,
                  data: frame,
                }),
              );
              timestamp += 20_000; // Opus frames are typically 20ms = 20000us
            } catch {
              // Skip bad frames
            }
          }
        }
      }
    } catch (e) {
      if (e instanceof Error && e.name !== "AbortError") {
        console.error("[Audio] Stream error:", e);
      }
    }
  }

  get isRecording(): boolean {
    return this.mediaRecorder?.state === "recording";
  }

  stop(): void {
    console.log("[WC] Stopping");

    this.abortController?.abort();
    this.abortController = null;

    this.audioAbortController?.abort();
    this.audioAbortController = null;

    if (this.decoder && this.decoder.state !== "closed") {
      try { this.decoder.close(); } catch { /* */ }
    }
    this.decoder = null;

    if (this.audioDecoder && this.audioDecoder.state !== "closed") {
      try { this.audioDecoder.close(); } catch { /* */ }
    }
    this.audioDecoder = null;

    if (this.audioContext) {
      this.audioContext.close();
      this.audioContext = null;
    }
    this.audioStartTime = 0;
    this.audioSampleOffset = 0;

    // Restore video element
    if (this.canvas?.parentElement && this.video) {
      this.canvas.parentElement.replaceChild(this.video, this.canvas);
    }
    this.canvas = null;
    this.ctx = null;
    this.configured = false;
    this.sps = null;
    this.pps = null;
    this.frameCount = 0;
  }
}
