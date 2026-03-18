import { useRef, useEffect, useCallback } from "react";
import { MsePlayer } from "../../lib/mse-player";
import { injectTouch, injectScroll, injectKey } from "../../lib/tauri-commands";
import { useSessionStore } from "../../stores/sessionStore";

interface VideoPlayerProps {
  streamUrl: string;
}

export function VideoPlayer({ streamUrl }: VideoPlayerProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const playerRef = useRef<MsePlayer | null>(null);
  const mountedRef = useRef(false);

  const session = useSessionStore((s) =>
    s.activeDeviceId ? s.sessions[s.activeDeviceId] ?? null : null,
  );

  const deviceId = session?.device_id;
  const screenWidth = session?.screen_width ?? 0;
  const screenHeight = session?.screen_height ?? 0;
  const audioCodec = session?.audio_codec ?? "opus";

  useEffect(() => {
    if (mountedRef.current) return;
    mountedRef.current = true;

    const container = containerRef.current;
    if (!container) return;

    // Create a video element for the player (it will be replaced by canvas internally)
    const video = document.createElement("video");
    video.className = "h-full rounded-lg shadow-2xl bg-neutral-900";
    video.style.cssText = "object-fit: contain; max-height: 100%; max-width: 100%;";
    video.autoplay = true;
    video.muted = true;
    video.playsInline = true;
    container.appendChild(video);

    const setPlayer = useSessionStore.getState().setPlayer;
    const player = new MsePlayer(video, streamUrl, audioCodec);
    playerRef.current = player;
    setPlayer(player);
    player.start().catch(console.error);

    return () => {
      mountedRef.current = false;
      player.stop();
      playerRef.current = null;
      setPlayer(null);
      // Clean up any remaining child elements
      while (container.firstChild) {
        container.removeChild(container.firstChild);
      }
    };
  }, [streamUrl]);

  /** Map mouse position to phone coordinates, clamping to screen bounds */
  const mapCoords = useCallback(
    (clientX: number, clientY: number, clamp = false): { x: number; y: number } | null => {
      const target = containerRef.current?.querySelector("canvas, video") as
        | HTMLCanvasElement
        | HTMLVideoElement
        | null;
      if (!target || !screenWidth || !screenHeight) return null;

      const rect = target.getBoundingClientRect();
      let phoneX = ((clientX - rect.left) / rect.width) * screenWidth;
      let phoneY = ((clientY - rect.top) / rect.height) * screenHeight;

      if (!clamp && (phoneX < 0 || phoneY < 0 || phoneX >= screenWidth || phoneY >= screenHeight)) {
        return null;
      }

      return { x: phoneX, y: phoneY };
    },
    [screenWidth, screenHeight],
  );

  const draggingRef = useRef(false);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (!deviceId) return;
      const pos = mapCoords(e.clientX, e.clientY);
      if (!pos) return;
      draggingRef.current = true;
      injectTouch(deviceId, "down", pos.x, pos.y, screenWidth, screenHeight);
    },
    [deviceId, screenWidth, screenHeight, mapCoords],
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!draggingRef.current || !deviceId) return;
      // Clamp to screen bounds so dragging outside the screen still sends edge position
      const pos = mapCoords(e.clientX, e.clientY, true);
      if (!pos) return;
      injectTouch(deviceId, "move", pos.x, pos.y, screenWidth, screenHeight);
    },
    [deviceId, screenWidth, screenHeight, mapCoords],
  );

  const handleMouseUp = useCallback(
    (e: React.MouseEvent) => {
      if (!draggingRef.current || !deviceId) return;
      draggingRef.current = false;
      const pos = mapCoords(e.clientX, e.clientY, true);
      if (!pos) return;
      injectTouch(deviceId, "up", pos.x, pos.y, screenWidth, screenHeight);
    },
    [deviceId, screenWidth, screenHeight, mapCoords],
  );

  // Listen on window for mousemove/mouseup during drag so we capture outside the canvas
  useEffect(() => {
    const onWindowMouseMove = (e: MouseEvent) => {
      if (!draggingRef.current || !deviceId) return;
      const pos = mapCoords(e.clientX, e.clientY, true);
      if (!pos) return;
      injectTouch(deviceId, "move", pos.x, pos.y, screenWidth, screenHeight);
    };
    const onWindowMouseUp = (e: MouseEvent) => {
      if (!draggingRef.current || !deviceId) return;
      draggingRef.current = false;
      const pos = mapCoords(e.clientX, e.clientY, true);
      if (!pos) return;
      injectTouch(deviceId, "up", pos.x, pos.y, screenWidth, screenHeight);
    };
    window.addEventListener("mousemove", onWindowMouseMove);
    window.addEventListener("mouseup", onWindowMouseUp);
    return () => {
      window.removeEventListener("mousemove", onWindowMouseMove);
      window.removeEventListener("mouseup", onWindowMouseUp);
    };
  }, [deviceId, screenWidth, screenHeight, mapCoords]);

  // Attach wheel listener natively with { passive: false } to allow preventDefault
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const onWheel = (e: WheelEvent) => {
      if (!deviceId) return;
      const pos = mapCoords(e.clientX, e.clientY);
      if (!pos) return;
      e.preventDefault();
      const vscroll = e.deltaY > 0 ? -1 : 1;
      const hscroll = e.deltaX > 0 ? -1 : e.deltaX < 0 ? 1 : 0;
      injectScroll(deviceId, pos.x, pos.y, screenWidth, screenHeight, hscroll, vscroll);
    };

    container.addEventListener("wheel", onWheel, { passive: false });
    return () => container.removeEventListener("wheel", onWheel);
  }, [deviceId, screenWidth, screenHeight, mapCoords]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (!deviceId) return;
      const keycode = browserKeyToAndroid(e.code);
      if (keycode !== null) {
        e.preventDefault();
        injectKey(deviceId, "down", keycode);
      }
    },
    [deviceId],
  );

  const handleKeyUp = useCallback(
    (e: React.KeyboardEvent) => {
      if (!deviceId) return;
      const keycode = browserKeyToAndroid(e.code);
      if (keycode !== null) {
        e.preventDefault();
        injectKey(deviceId, "up", keycode);
      }
    },
    [deviceId],
  );

  return (
    <div
      ref={containerRef}
      className="relative h-full w-full flex items-center justify-center overflow-hidden"
      tabIndex={0}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onKeyDown={handleKeyDown}
      onKeyUp={handleKeyUp}
      onContextMenu={(e) => e.preventDefault()}
      style={{ outline: "none" }}
    />
  );
}

/** Maps browser KeyboardEvent.code to Android KEYCODE values */
function browserKeyToAndroid(code: string): number | null {
  const map: Record<string, number> = {
    Backspace: 67,
    Enter: 66,
    Escape: 4, // BACK
    Tab: 61,
    Space: 62,
    ArrowUp: 19,
    ArrowDown: 20,
    ArrowLeft: 21,
    ArrowRight: 22,
    Delete: 112,
    Home: 3, // HOME
    // Letters
    KeyA: 29, KeyB: 30, KeyC: 31, KeyD: 32, KeyE: 33, KeyF: 34, KeyG: 35,
    KeyH: 36, KeyI: 37, KeyJ: 38, KeyK: 39, KeyL: 40, KeyM: 41, KeyN: 42,
    KeyO: 43, KeyP: 44, KeyQ: 45, KeyR: 46, KeyS: 47, KeyT: 48, KeyU: 49,
    KeyV: 50, KeyW: 51, KeyX: 52, KeyY: 53, KeyZ: 54,
    // Numbers
    Digit0: 7, Digit1: 8, Digit2: 9, Digit3: 10, Digit4: 11,
    Digit5: 12, Digit6: 13, Digit7: 14, Digit8: 15, Digit9: 16,
  };
  return map[code] ?? null;
}
