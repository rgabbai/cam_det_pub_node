//! Camera Modules
//!

use rscam::{Camera, Config};
use std::fs;
use std::io::Write;
use std::path::Path;


pub struct CameraConfig {
    pub path: String,
}


pub struct UsbCamera {
    //Camera instance
    camera: Camera, 
    //Camera configuration
    config: CameraConfig,
}

fn open_camera() -> Result<Camera, String> {
    let device_paths = ["/dev/video5", "/dev/video0"]; // Add more paths if necessary

    for path in &device_paths {
        if Path::new(path).exists() {
            match Camera::new(path) {
                Ok(camera) => return Ok(camera),
                Err(_) => continue,
            }
        }
    }

    Err("Unable to open camera on any of the provided device paths".to_string())
}

impl UsbCamera {
    //set the camera 
    pub fn new() -> Self {
        // For Rockpi camera registered as video0
       //let mut camera = Camera::new("/dev/video0").expect("Can't open the camera ");
       // For asus - USB Camera is registered as video5  
       //let mut camera = Camera::new("/dev/video5").expect("Can't open the camera ");
       let mut camera = open_camera().expect("Can't open the camera");

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

