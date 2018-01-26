use super::reqwest;
use super::goblin;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use std::ffi::CStr;
use super::error::PdbError;
use failure::Error;
use super::indicatif::ProgressBar;

pub struct PdbDownloader {
    filename: String
}

impl PdbDownloader {
    pub fn new(filename: String) -> PdbDownloader {
        PdbDownloader{
            filename: filename
        }
    }

    fn download_pdb(&self) -> Result<reqwest::Response, PdbError> {
        let url = self.generate_url();
        let resp = match reqwest::get(&url) {
            Err(err) => return Err(PdbError::DownloadFailed(err.to_string())),
            Ok(resp) => resp
        };

        if !resp.status().is_success() {
            return Err(PdbError::StatusError(format!("{}", resp.status())))
        }

        Ok(resp)
    }

    pub fn download(&self) -> Result<(), Error> {

        let progress = ProgressBar::new_spinner();
        progress.set_message(&"Downloading data..");
        let mut response = self.download_pdb()?;

        progress.set_message(&"Downloading data....");
        let filename = Path::new(&self.filename).file_stem().unwrap();

        progress.set_message(&"Downloading data......");
        let mut pdb_filename = String::from(filename.to_str().unwrap());

        progress.set_message(&"Downloading data........");
        pdb_filename.push_str(".pdb");

        progress.set_message(&"Downloading data..........");
        let path = Path::new(&pdb_filename);

        progress.set_message(&"Downloading data............");
        let mut fd = File::create(path)?;

        progress.set_message(&"Downloading data..............");
        let mut buf: Vec<u8> = vec![];
        response.copy_to(&mut buf)?;

        println!("Download complete !");
        fd.write_all(&buf)?;

        println!("");
        Ok(())

    }

    fn generate_url(&self) -> String {
        let mut fd = File::open(Path::new(&self.filename)).expect("Can't open file");
        let buffer = { let mut v = Vec::new(); fd.read_to_end(&mut v).unwrap(); v};
        let res = goblin::Object::parse(&buffer).expect("Can't parse PE");

        if let goblin::Object::PE(pe) = res {
            let codeview_info = pe.debug_data.unwrap().codeview_pdb70_debug_info.unwrap();

            let file = codeview_info.filename;
            let age = codeview_info.age;
            let guid = codeview_info.signature;

            let guid_str = format!("{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:X}",
                                    guid[3], guid[2], guid[1], guid[0], guid[5], guid[4], guid[7], guid[6],
                                    guid[8], guid[9], guid[10], guid[11], guid[12], guid[13], guid[14], guid[15], age);

            let file = CStr::from_bytes_with_nul(file).unwrap().to_str().unwrap();

            let url = format!("{}/{}/{}/{}", "https://msdl.microsoft.com/download/symbols", file, guid_str, file);

            return url;
        }

        panic!("can't generate pdb from file");
    }
}

#[cfg(test)]
mod tests {
    use super::PdbDownloader;

    #[test]
    fn test_nt_pdb_is_correct() {
        let pdb = PdbDownloader::new("c:\\windows\\system32\\ntoskrnl.exe".to_string());
        assert_eq!(pdb.generate_url(),
       "https://msdl.microsoft.com/download/symbols/ntkrnlmp.pdb/31C51B7D1C2545A88F69E13FC73E68941/ntkrnlmp.pdb")
    }

    #[test]
    fn test_download_is_working() {
        let pdb = PdbDownloader::new("c:\\windows\\system32\\ntoskrnl.exe".to_string());
        assert!(pdb.download().is_ok());
    }
}
