use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use rclrust::{qos::QoSProfile, rclrust_info};
use rclrust_msg::std_msgs::msg::String as String_;

use serde::{Serialize, Deserialize};
use serde_json;

pub mod camera;
pub mod obj_detect;

const TOPIC_NAME: &str = "detect";


#[derive(Serialize, Deserialize, Debug)]
struct BoxCor(f32, f32, f32, f32);

#[derive(Serialize, Deserialize, Debug)]
struct DetObj {
    box_location: BoxCor,
    otype: String,
    prob: f32,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Detect publisher node start");
    // take a pic
    let cam = camera::UsbCamera::new();
    //let mut detect_res :String = String::new();
   
   
    let ctx = rclrust::init()?;
    let mut node = ctx.create_node("cam_det_publisher")?;
    let logger = node.logger();
    let count = AtomicUsize::new(0);
    let publisher = node.create_publisher::<String_>(TOPIC_NAME, &QoSProfile::default())?;

    let _timer = node.create_wall_timer(Duration::from_millis(2000), move || {
        count.fetch_add(1, Ordering::Relaxed);

        
        // capture image 
        let _ = cam.take_pic();
        // Detect
        println!("Detection starts!");
        let detect_res = obj_detect::detect("image.jpg");
        //process string to DetObj format

        let mut detected_objects: Vec<DetObj> = Vec::new();

        for detection in &detect_res {
            let obj = DetObj {
                box_location: BoxCor(detection.0, detection.1, detection.2, detection.3),
                otype: detection.4.to_string(),
                prob: detection.5,
            };
            detected_objects.push(obj);
        }

        let serialized_data = match serde_json::to_string(&detected_objects) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to serialize detected data: {}", e);
                return;  // Don't proceed if serialization fails
            }
        };

        let message = String_ {
            //data: format!("{:?} {}",detect_res,count.load(Ordering::Relaxed)),
            data: serialized_data,
        };
        rclrust_info!(logger, "Publishing: '{}'", message.data);
        publisher.publish(&message).unwrap();
    })?;

    node.wait();

    Ok(())
}
