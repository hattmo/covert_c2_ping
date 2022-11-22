use anyhow::Result;
use std::{io::ErrorKind, process::Command, thread::sleep, time::Duration};
pub struct Rules {}
impl Drop for Rules {
    #[tracing::instrument(name = "teardown_rules", skip_all)]
    fn drop(&mut self) {
        tracing::info!("Clearing rules");
        if Command::new("nft")
            .arg("delete table ip covert_table")
            .status()
            .is_err()
        {
            tracing::info!("Failed to clear rules");
        } else {
            tracing::info!("Rules cleared");
        };
    }
}
impl Rules {
    #[tracing::instrument(name = "setup_rules")]
    pub fn new() -> Result<Self> {
        tracing::info!("Setting up rules");
        Command::new("nft")
        .arg("add table ip covert_table")
        .status().and_then(|code|{
            if code.success(){
                Command::new("nft").arg("add chain ip covert_table covert_chain { type filter hook input priority -250 ; policy accept ; }").status()
            }else{
                Err(std::io::Error::new(ErrorKind::Other, "Failed: nft add table ip covert_table"))
            }
        }).and_then(|code|{
            if code.success(){
                Command::new("nft").arg("flush chain ip covert_table covert_chain").status()
            }else{
                Err(std::io::Error::new(ErrorKind::Other, "Failed: nft add chain ip covert_table covert_chain { type filter hook input priority -250 ; policy accept ; }"))
            }
        }).and_then(|code|{
            if code.success(){
                Command::new("nft").arg("add rule ip covert_table covert_chain icmp type { echo-request } queue num 42 bypass").status()
            }else{
                Err(std::io::Error::new(ErrorKind::Other, "Failed: nft flush chain ip covert_table cover_chain"))
            }
        }).and_then(|code|{
            if code.success(){
                std::io::Result::Ok(())
            }else{
                Err(std::io::Error::new(ErrorKind::Other, "Failed: nft add rule ip covert_table covert_chain icmp type { echo-request } queue num 42 bypass"))
            }
        })?;
        sleep(Duration::from_secs(1));
        tracing::info!("Rules setup");
        Ok(Rules {})
    }
}
