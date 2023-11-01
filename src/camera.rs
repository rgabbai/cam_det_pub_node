//! Camera Modules
//!

use rscam::{Camera, Config};
use std::fs;
use std::io::Write;


pub struct CameraConfig {
    pub path: String,
}


pub struct UsbCamera {
    //Camera instance
    camera: Camera, 
    //Camera configuration
    config: CameraConfig,
}

impl UsbCamera {
    //set the camera 
    pub fn new() -> Self {
       let mut camera = Camera::new("/dev/video0").expect("Can't open the camera ");
        // start the camera
        camera.start(&Config {
          interval: (1,30),
          resolution: (640,360),
          format: b"MJPG",
          nbuffers: 1,
          ..Default::default()
        })
        .expect("can't start camera capture");

        let config = CameraConfig{path: String::from("image.jpg")};

        Self{ camera , config}
    }
    // Capture a frame and save it to a File
    pub fn take_pic(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        for _ in 0..3 {
            let _ = self.camera.capture(); // Grab a frame to reduce delay.
        }
        let frame = self.camera.capture()?; // get picture

        // Save the original image to the specified file path.
        let mut file = fs::File::create(self.config.path.clone())?;
        file.write_all(&frame[..])?;
        // See if I can convert vector back to image
        //let frame_vector = frame.to_vec();
        //let mut file = fs::File::create("test_image.jpg")?;
        //file.write_all(&frame_vector)?;

        Ok(frame.to_vec())
    }

}

