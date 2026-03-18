/// Minimal FairPlay stub responses for AirPlay handshake.
/// Full FairPlay is NOT required for basic screen mirroring.
/// These stubs allow the handshake to complete without DRM support.

pub fn handle_fp_setup(request_body: &[u8]) -> Vec<u8> {
    // FairPlay setup has multiple stages identified by the first byte
    if request_body.is_empty() {
        return vec![0u8; 4];
    }

    match request_body[0] {
        // Stage 1: FairPlay challenge
        1 => {
            tracing::debug!("FairPlay stage 1 (challenge), {} bytes", request_body.len());
            // Return a minimal response that allows the handshake to continue
            // This won't work for DRM content but allows basic mirroring
            vec![2, 0, 0, 0]
        }
        // Stage 3: FairPlay verify
        3 => {
            tracing::debug!("FairPlay stage 3 (verify), {} bytes", request_body.len());
            vec![4, 0, 0, 0]
        }
        _ => {
            tracing::debug!(
                "FairPlay unknown stage {}, {} bytes",
                request_body[0],
                request_body.len()
            );
            vec![0u8; 4]
        }
    }
}
