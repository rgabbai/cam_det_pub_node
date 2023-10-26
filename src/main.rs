use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use rclrust::{qos::QoSProfile, rclrust_info};
use rclrust_msg::std_msgs::msg::String as String_;


pub mod camera;
pub mod obj_detect;



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Detect publisher node start");
    // take a pic
    let cam = camera::UsbCamera::new();
    //let mut detect_res :String = String::new();
   
   
    let ctx = rclrust::init()?;
    let mut node = ctx.create_node("minimal_publisher")?;
    let logger = node.logger();
    let count = AtomicUsize::new(0);
    let publisher = node.create_publisher::<String_>("topic", &QoSProfile::default())?;

    let _timer = node.create_wall_timer(Duration::from_millis(2000), move || {
        count.fetch_add(1, Ordering::Relaxed);

        
        // capture image 
        let _ = cam.take_pic();
        // Detect
        println!("Detection starts!");
        let detect_res = obj_detect::detect("image.jpg");



        let message = String_ {
            data: format!("{:?} {}",detect_res,count.load(Ordering::Relaxed)),
        };
        rclrust_info!(logger, "Publishing: '{}'", message.data);
        publisher.publish(&message).unwrap();
    })?;

    node.wait();

    Ok(())
}
