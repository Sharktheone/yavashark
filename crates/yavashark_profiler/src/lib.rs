
struct Profile;

trait ProfileWriter {
    fn write_profile(&mut self, profile: Profile) -> Result<Vec<u8>, std::io::Error>;
}