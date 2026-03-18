use bytes::{BufMut, Bytes, BytesMut};
use tracing::debug;

/// Parses H.264 NAL units and produces fragmented MP4 (fMP4) segments.
///
/// The fMP4 format is used because it can be consumed by the browser's
/// MediaSource Extensions (MSE) API, enabling hardware-accelerated decoding
/// in the WebView without any raw pixel copies.
pub struct FMp4Muxer {
    sequence_number: u32,
    sps: Option<Bytes>,
    pps: Option<Bytes>,
    width: u32,
    height: u32,
    timescale: u32,
    frame_duration: u32,
    decode_time: u64,
    init_segment: Option<Bytes>,
}

impl FMp4Muxer {
    pub fn new() -> Self {
        Self {
            sequence_number: 1,
            sps: None,
            pps: None,
            width: 0,
            height: 0,
            timescale: 90000,
            frame_duration: 1500, // 90000 / 60fps
            decode_time: 0,
            init_segment: None,
        }
    }

    /// Process a complete frame (Annex-B format, may contain multiple NAL units).
    /// Extracts SPS/PPS for init segment, converts video NALs to AVC format,
    /// and returns an fMP4 media segment.
    pub fn process_frame(&mut self, annex_b_data: &[u8]) -> Option<Bytes> {
        if annex_b_data.is_empty() {
            return None;
        }

        let nals = Self::split_nals(annex_b_data);
        let mut video_nals: Vec<&[u8]> = Vec::new();
        let mut is_keyframe = false;

        for nal in &nals {
            if nal.is_empty() {
                continue;
            }
            let nal_type = nal[0] & 0x1F;

            match nal_type {
                7 => {
                    // SPS
                    self.sps = Some(Bytes::copy_from_slice(nal));
                    self.parse_sps(nal);
                    debug!("Got SPS: {}x{}", self.width, self.height);
                    self.try_create_init_segment();
                }
                8 => {
                    // PPS
                    self.pps = Some(Bytes::copy_from_slice(nal));
                    debug!("Got PPS");
                    self.try_create_init_segment();
                }
                5 => {
                    // IDR slice
                    is_keyframe = true;
                    video_nals.push(nal);
                }
                1 => {
                    // Non-IDR slice
                    video_nals.push(nal);
                }
                6 => {
                    // SEI - skip
                }
                _ => {
                    video_nals.push(nal);
                }
            }
        }

        if video_nals.is_empty() || self.init_segment.is_none() {
            return None;
        }

        // Convert all video NALs to AVC format (length-prefixed) in a single mdat
        Some(self.create_media_segment_multi(&video_nals, is_keyframe))
    }

    /// Get the initialization segment (ftyp + moov).
    /// Must be sent to the browser before any media segments.
    pub fn init_segment(&self) -> Option<Bytes> {
        self.init_segment.clone()
    }

    /// Parse SPS to extract width and height
    fn parse_sps(&mut self, sps: &[u8]) {
        // Simplified SPS parsing - extract width/height from common positions
        // A full implementation would use an exp-golomb decoder
        if sps.len() < 8 {
            return;
        }
        // For now, use defaults that will be updated when the actual resolution is known
        if self.width == 0 {
            self.width = 1920;
            self.height = 1080;
        }
    }

    /// Try to create the initialization segment once we have both SPS and PPS
    fn try_create_init_segment(&mut self) {
        if let (Some(sps), Some(pps)) = (&self.sps, &self.pps) {
            let init = self.build_init_segment(sps, pps);
            self.init_segment = Some(init);
            debug!("Created fMP4 init segment");
        }
    }

    /// Build the ftyp + moov boxes for the initialization segment
    fn build_init_segment(&self, sps: &[u8], pps: &[u8]) -> Bytes {
        let mut buf = BytesMut::with_capacity(512);

        // ftyp box
        Self::write_ftyp(&mut buf);

        // moov box
        self.write_moov(&mut buf, sps, pps);

        buf.freeze()
    }

    /// Create a media segment (moof + mdat) for multiple NALs forming one frame
    fn create_media_segment_multi(&mut self, nals: &[&[u8]], is_keyframe: bool) -> Bytes {
        // Calculate total sample size: each NAL gets a 4-byte length prefix
        let sample_size: u32 = nals.iter()
            .map(|nal| 4 + nal.len() as u32)
            .sum();

        let mut buf = BytesMut::with_capacity(sample_size as usize + 256);

        let seq = self.sequence_number;
        self.sequence_number += 1;

        // moof box
        self.write_moof(&mut buf, seq, sample_size, is_keyframe);

        // mdat box with all NALs in AVC format (length-prefixed)
        let mdat_size = 8 + sample_size;
        buf.put_u32(mdat_size);
        buf.put_slice(b"mdat");
        for nal in nals {
            buf.put_u32(nal.len() as u32); // 4-byte length prefix
            buf.put_slice(nal);
        }

        self.decode_time += self.frame_duration as u64;

        buf.freeze()
    }

    /// Split Annex-B byte stream into individual NAL units
    fn split_nals(data: &[u8]) -> Vec<&[u8]> {
        let mut nals = Vec::new();
        let len = data.len();
        let mut i = 0;

        while i < len {
            // Find start code
            let start;
            if i + 4 <= len && data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 0 && data[i + 3] == 1 {
                start = i + 4;
            } else if i + 3 <= len && data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1 {
                start = i + 3;
            } else {
                i += 1;
                continue;
            }

            // Find next start code
            let mut end = start;
            while end < len {
                if end + 4 <= len && data[end] == 0 && data[end + 1] == 0 && data[end + 2] == 0 && data[end + 3] == 1 {
                    break;
                }
                if end + 3 <= len && data[end] == 0 && data[end + 1] == 0 && data[end + 2] == 1 {
                    break;
                }
                end += 1;
            }

            if end > start {
                nals.push(&data[start..end]);
            }
            i = end;
        }

        if nals.is_empty() && !data.is_empty() {
            nals.push(data);
        }

        nals
    }

    fn write_ftyp(buf: &mut BytesMut) {
        let ftyp_data = b"isom\x00\x00\x02\x00isomiso6mp41";
        let size = 8 + ftyp_data.len() as u32;
        buf.put_u32(size);
        buf.put_slice(b"ftyp");
        buf.put_slice(ftyp_data);
    }

    fn write_moov(&self, buf: &mut BytesMut, sps: &[u8], pps: &[u8]) {
        let moov_start = buf.len();
        buf.put_u32(0); // placeholder for size
        buf.put_slice(b"moov");

        // mvhd (movie header)
        self.write_mvhd(buf);

        // trak (track)
        self.write_trak(buf, sps, pps);

        // mvex (movie extends - required for fMP4)
        self.write_mvex(buf);

        // Update moov size
        let moov_size = (buf.len() - moov_start) as u32;
        let moov_start_bytes = moov_size.to_be_bytes();
        buf[moov_start..moov_start + 4].copy_from_slice(&moov_start_bytes);
    }

    fn write_mvhd(&self, buf: &mut BytesMut) {
        let size: u32 = 108;
        buf.put_u32(size);
        buf.put_slice(b"mvhd");
        buf.put_u32(0); // version + flags
        buf.put_u32(0); // creation_time
        buf.put_u32(0); // modification_time
        buf.put_u32(self.timescale); // timescale
        buf.put_u32(0); // duration
        buf.put_u32(0x00010000); // rate (1.0)
        buf.put_u16(0x0100); // volume (1.0)
        buf.put_slice(&[0u8; 10]); // reserved
        // Matrix (identity)
        for &val in &[
            0x00010000u32,
            0,
            0,
            0,
            0x00010000,
            0,
            0,
            0,
            0x40000000,
        ] {
            buf.put_u32(val);
        }
        buf.put_slice(&[0u8; 24]); // pre-defined
        buf.put_u32(2); // next_track_ID
    }

    fn write_trak(&self, buf: &mut BytesMut, sps: &[u8], pps: &[u8]) {
        let trak_start = buf.len();
        buf.put_u32(0);
        buf.put_slice(b"trak");

        self.write_tkhd(buf);
        self.write_mdia(buf, sps, pps);

        let trak_size = (buf.len() - trak_start) as u32;
        buf[trak_start..trak_start + 4].copy_from_slice(&trak_size.to_be_bytes());
    }

    fn write_tkhd(&self, buf: &mut BytesMut) {
        let size: u32 = 92;
        buf.put_u32(size);
        buf.put_slice(b"tkhd");
        buf.put_u32(0x00000003); // version=0, flags=enabled+in_movie
        buf.put_u32(0); // creation_time
        buf.put_u32(0); // modification_time
        buf.put_u32(1); // track_ID
        buf.put_u32(0); // reserved
        buf.put_u32(0); // duration
        buf.put_slice(&[0u8; 8]); // reserved
        buf.put_u16(0); // layer
        buf.put_u16(0); // alternate_group
        buf.put_u16(0); // volume (0 for video)
        buf.put_u16(0); // reserved
        // Matrix (identity)
        for &val in &[
            0x00010000u32,
            0,
            0,
            0,
            0x00010000,
            0,
            0,
            0,
            0x40000000,
        ] {
            buf.put_u32(val);
        }
        buf.put_u32(self.width << 16); // width as fixed-point
        buf.put_u32(self.height << 16); // height as fixed-point
    }

    fn write_mdia(&self, buf: &mut BytesMut, sps: &[u8], pps: &[u8]) {
        let mdia_start = buf.len();
        buf.put_u32(0);
        buf.put_slice(b"mdia");

        self.write_mdhd(buf);
        self.write_hdlr(buf);
        self.write_minf(buf, sps, pps);

        let mdia_size = (buf.len() - mdia_start) as u32;
        buf[mdia_start..mdia_start + 4].copy_from_slice(&mdia_size.to_be_bytes());
    }

    fn write_mdhd(&self, buf: &mut BytesMut) {
        let size: u32 = 32;
        buf.put_u32(size);
        buf.put_slice(b"mdhd");
        buf.put_u32(0); // version + flags
        buf.put_u32(0); // creation_time
        buf.put_u32(0); // modification_time
        buf.put_u32(self.timescale); // timescale
        buf.put_u32(0); // duration
        buf.put_u16(0x55C4); // language (und)
        buf.put_u16(0); // pre-defined
    }

    fn write_hdlr(&self, buf: &mut BytesMut) {
        let size: u32 = 45;
        buf.put_u32(size);
        buf.put_slice(b"hdlr");
        buf.put_u32(0); // version + flags
        buf.put_u32(0); // pre-defined
        buf.put_slice(b"vide"); // handler_type
        buf.put_slice(&[0u8; 12]); // reserved
        buf.put_slice(b"VideoHandler\0"); // name
    }

    fn write_minf(&self, buf: &mut BytesMut, sps: &[u8], pps: &[u8]) {
        let minf_start = buf.len();
        buf.put_u32(0);
        buf.put_slice(b"minf");

        // vmhd
        buf.put_u32(20);
        buf.put_slice(b"vmhd");
        buf.put_u32(1); // version=0, flags=1
        buf.put_u16(0); // graphicsmode
        buf.put_slice(&[0u8; 6]); // opcolor

        // dinf + dref
        buf.put_u32(36);
        buf.put_slice(b"dinf");
        buf.put_u32(28);
        buf.put_slice(b"dref");
        buf.put_u32(0); // version + flags
        buf.put_u32(1); // entry_count
        buf.put_u32(12);
        buf.put_slice(b"url ");
        buf.put_u32(1); // self-contained flag

        // stbl
        self.write_stbl(buf, sps, pps);

        let minf_size = (buf.len() - minf_start) as u32;
        buf[minf_start..minf_start + 4].copy_from_slice(&minf_size.to_be_bytes());
    }

    fn write_stbl(&self, buf: &mut BytesMut, sps: &[u8], pps: &[u8]) {
        let stbl_start = buf.len();
        buf.put_u32(0);
        buf.put_slice(b"stbl");

        // stsd (sample description)
        self.write_stsd(buf, sps, pps);

        // stts (empty, required)
        buf.put_u32(16);
        buf.put_slice(b"stts");
        buf.put_u32(0);
        buf.put_u32(0);

        // stsc (empty, required)
        buf.put_u32(16);
        buf.put_slice(b"stsc");
        buf.put_u32(0);
        buf.put_u32(0);

        // stsz (empty, required)
        buf.put_u32(20);
        buf.put_slice(b"stsz");
        buf.put_u32(0);
        buf.put_u32(0);
        buf.put_u32(0);

        // stco (empty, required)
        buf.put_u32(16);
        buf.put_slice(b"stco");
        buf.put_u32(0);
        buf.put_u32(0);

        let stbl_size = (buf.len() - stbl_start) as u32;
        buf[stbl_start..stbl_start + 4].copy_from_slice(&stbl_size.to_be_bytes());
    }

    fn write_stsd(&self, buf: &mut BytesMut, sps: &[u8], pps: &[u8]) {
        let stsd_start = buf.len();
        buf.put_u32(0);
        buf.put_slice(b"stsd");
        buf.put_u32(0); // version + flags
        buf.put_u32(1); // entry_count

        // avc1 sample entry
        let avc1_start = buf.len();
        buf.put_u32(0);
        buf.put_slice(b"avc1");
        buf.put_slice(&[0u8; 6]); // reserved
        buf.put_u16(1); // data_reference_index
        buf.put_slice(&[0u8; 16]); // pre-defined + reserved
        buf.put_u16(self.width as u16);
        buf.put_u16(self.height as u16);
        buf.put_u32(0x00480000); // horiz resolution 72dpi
        buf.put_u32(0x00480000); // vert resolution 72dpi
        buf.put_u32(0); // reserved
        buf.put_u16(1); // frame_count
        buf.put_slice(&[0u8; 32]); // compressorname
        buf.put_u16(0x0018); // depth
        buf.put_i16(-1); // pre-defined

        // avcC box (AVC decoder configuration)
        let avcc_start = buf.len();
        buf.put_u32(0); // placeholder for size
        buf.put_slice(b"avcC");
        buf.put_u8(1); // configurationVersion
        buf.put_u8(if sps.len() > 1 { sps[1] } else { 0x64 }); // AVCProfileIndication
        buf.put_u8(if sps.len() > 2 { sps[2] } else { 0x00 }); // profile_compatibility
        buf.put_u8(if sps.len() > 3 { sps[3] } else { 0x1F }); // AVCLevelIndication
        buf.put_u8(0xFF); // lengthSizeMinusOne = 3 (4-byte NAL lengths)
        buf.put_u8(0xE1); // numOfSequenceParameterSets = 1
        buf.put_u16(sps.len() as u16);
        buf.put_slice(sps);
        buf.put_u8(1); // numOfPictureParameterSets
        buf.put_u16(pps.len() as u16);
        buf.put_slice(pps);

        // Patch avcC size from actual bytes written
        let avcc_size = (buf.len() - avcc_start) as u32;
        buf[avcc_start..avcc_start + 4].copy_from_slice(&avcc_size.to_be_bytes());

        let avc1_size = (buf.len() - avc1_start) as u32;
        buf[avc1_start..avc1_start + 4].copy_from_slice(&avc1_size.to_be_bytes());

        let stsd_size = (buf.len() - stsd_start) as u32;
        buf[stsd_start..stsd_start + 4].copy_from_slice(&stsd_size.to_be_bytes());
    }

    fn write_mvex(&self, buf: &mut BytesMut) {
        let mvex_start = buf.len();
        buf.put_u32(0);
        buf.put_slice(b"mvex");

        // trex (track extends)
        buf.put_u32(32);
        buf.put_slice(b"trex");
        buf.put_u32(0); // version + flags
        buf.put_u32(1); // track_ID
        buf.put_u32(1); // default_sample_description_index
        buf.put_u32(0); // default_sample_duration
        buf.put_u32(0); // default_sample_size
        buf.put_u32(0); // default_sample_flags

        let mvex_size = (buf.len() - mvex_start) as u32;
        buf[mvex_start..mvex_start + 4].copy_from_slice(&mvex_size.to_be_bytes());
    }

    fn write_moof(&self, buf: &mut BytesMut, sequence_number: u32, sample_size: u32, is_keyframe: bool) {
        let moof_start = buf.len();
        buf.put_u32(0);
        buf.put_slice(b"moof");

        // mfhd (movie fragment header)
        buf.put_u32(16);
        buf.put_slice(b"mfhd");
        buf.put_u32(0); // version + flags
        buf.put_u32(sequence_number);

        // traf (track fragment)
        let traf_start = buf.len();
        buf.put_u32(0);
        buf.put_slice(b"traf");

        // tfhd (track fragment header)
        buf.put_u32(16);
        buf.put_slice(b"tfhd");
        buf.put_u32(0x020000); // flags: default-base-is-moof
        buf.put_u32(1); // track_ID

        // tfdt (track fragment decode time)
        buf.put_u32(20);
        buf.put_slice(b"tfdt");
        buf.put_u32(0x01000000); // version=1
        buf.put_u64(self.decode_time);

        // trun (track run)
        // flags: data-offset-present | first-sample-flags-present | sample-size-present | sample-duration-present
        let trun_flags: u32 = 0x000001 | 0x000004 | 0x000100 | 0x000200;
        let trun_start = buf.len();
        buf.put_u32(0); // placeholder for size
        buf.put_slice(b"trun");
        buf.put_u32(trun_flags); // version + flags
        buf.put_u32(1); // sample_count

        // data_offset (flag 0x001)
        let data_offset_pos = buf.len();
        buf.put_u32(0); // placeholder - patched after moof size is known

        // first_sample_flags (flag 0x004)
        let sample_flags = if is_keyframe {
            0x02000000 // sample_depends_on=2 (no dependencies = keyframe)
        } else {
            0x01010000 // sample_depends_on=1, sample_is_non_sync=1
        };
        buf.put_u32(sample_flags);

        // per-sample: duration (flag 0x100) + size (flag 0x200)
        buf.put_u32(self.frame_duration);
        buf.put_u32(sample_size);

        // Patch trun size
        let trun_size = (buf.len() - trun_start) as u32;
        buf[trun_start..trun_start + 4].copy_from_slice(&trun_size.to_be_bytes());

        let traf_size = (buf.len() - traf_start) as u32;
        buf[traf_start..traf_start + 4].copy_from_slice(&traf_size.to_be_bytes());

        let moof_size = (buf.len() - moof_start) as u32;
        buf[moof_start..moof_start + 4].copy_from_slice(&moof_size.to_be_bytes());

        // Patch data_offset: offset from moof start to mdat payload
        let data_offset = moof_size + 8; // +8 for mdat header
        buf[data_offset_pos..data_offset_pos + 4].copy_from_slice(&data_offset.to_be_bytes());
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_resolution(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_fps(&mut self, fps: u32) {
        if fps > 0 {
            self.frame_duration = self.timescale / fps;
        }
    }
}
