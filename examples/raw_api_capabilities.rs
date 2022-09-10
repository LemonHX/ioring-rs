use ioring_rs::windows::{win_ring_capabilities, win_ring_query_capabilities};

fn main() -> std::io::Result<()> {
    unsafe {
        let mut cap: win_ring_capabilities = std::mem::zeroed();
        win_ring_query_capabilities(&mut cap);

        println!("IoRing Version: {}", cap.IoRingVersion);
        println!("Max opcode: {}", cap.MaxOpCode);
        println!("Supported flags: {}", cap.FlagsSupported);
        println!("Submission queue size: {}", cap.SubmissionQueueSize);
        println!("Completion queue size: {}", cap.CompletionQueueSize);

        Ok(())
    }
}
