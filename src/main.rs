use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime,Duration},
};

use rclrust::{qos::QoSProfile, rclrust_info};
use rclrust_msg::std_msgs::msg::String as String_;

use serde::{Serialize, Deserialize};
use serde_json;

//image topic 
//use rclrust_msg::sensor_msgs::msg::Image as ImageMsg;
use rclrust_msg::sensor_msgs::msg::CompressedImage as CompressedImageMsg;

//use rclrust_msg::std_msgs::msg::Header;

pub mod camera;
pub mod obj_detect;
pub mod estimation;

const TOPIC_NAME: &str = "detect";
const FPS: u64 = 1; // Frames per second
const SECONDS_PER_MINUTE: u64 = 60;
const MILLISECONDS_PER_SECOND: u64 = 1000;

#[derive(Serialize, Deserialize, Debug)]
struct BoxCor(f32, f32, f32, f32);

#[derive(Serialize, Deserialize, Debug)]
struct DetObj {
    box_location: BoxCor,
    otype: String,
    prob: f32,
    dist: f64,
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
    let publisher = node.create_publisher::<String_>(TOPIC_NAME, &QoSProfile::default())?;   // detection meta data publisher
    //let image_publisher = node.create_publisher::<ImageMsg>("camera_image", &QoSProfile::default())?; // actual image publisher
    let image_publisher = node.create_publisher::<CompressedImageMsg>("Compressed_camera_image", &QoSProfile::default())?;


    let period_ms: u64 = MILLISECONDS_PER_SECOND / FPS;
    println!(">FPS:{FPS} period [ms]:{period_ms}");

    let _timer = node.create_wall_timer(Duration::from_millis(period_ms), move || {
        count.fetch_add(1, Ordering::Relaxed);

        
        // capture image 
        let image_data = match cam.take_pic() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to capture image: {}", e);
                return; // Decide how to handle the error
            }
        };
        // TODO do msg conversion it in parallel to detection stage
        
        // ROS publisher section
        // Send MSG Topic of type: CompressedImageMsg
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        let stamp = rclrust_msg::builtin_interfaces::msg::Time {
            sec: now.as_secs() as i32,
            nanosec: now.subsec_nanos() as u32,
        };

        let image_message = CompressedImageMsg {
            header: rclrust_msg::std_msgs::msg::Header {
                stamp: stamp, 
                frame_id: "camera".to_string(),
                ..Default::default()
            },
            format: "jpeg".to_string(),  // For JPEG/MJPEG format
            data: image_data,

        };

            // Publish the image
         match image_publisher.publish(&image_message) {
            Ok(_) => rclrust_info!(logger, "Image published successfully."),
            Err(e) => eprintln!("Failed to publish image: {}", e),
        };

        // Detect stage
        println!("Detection starts!");
        let detect_res = obj_detect::detect("image.jpg");
        //process string to DetObj format

        let mut detected_objects: Vec<DetObj> = Vec::new();
        // Estimate Pylon distance in cm 
        // Given data points


        for detection in &detect_res {
            let pixel_height:f64 = (detection.3 - detection.1).into(); 
            println!("Pixel hieght:{}",pixel_height);
            let obj = DetObj {
                box_location: BoxCor(detection.0, detection.1, detection.2, detection.3),
                otype: detection.4.to_string(),
                prob: detection.5,
                dist: estimation::estimate_distance(pixel_height),
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
        
 

        // Publish detection data 
        
        let mut  message = String_ {
            //data: format!("{:?} {}",detect_res,count.load(Ordering::Relaxed)),
            data: serialized_data,
        };
        // Check if detection found something otherwise send nothing found msg msg 
        if message.data == "[]" {
            //println!("No detection");
            message.data = "[{\"box_location\":[0.0,0.0,0.0,0.0],\"otype\":\"nothing\",\"prob\":1.0,\"dist\":0.0}]".to_string();
        }

        rclrust_info!(logger, "Publishing: '{}'", message.data);
        publisher.publish(&message).unwrap();
    })?;

    node.wait();

    Ok(())
}
