use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime,Duration},
};
use image::{DynamicImage, GenericImageView, ImageOutputFormat,imageops::FilterType, ColorType};
use std::io::Cursor;

use std::env;
use std::process;
use clap::{App, Arg}; // handle arguments 

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
const FPS: f32 = 0.3; // Frames per second
const SECONDS_PER_MINUTE: f32 = 60.0;
const MILLISECONDS_PER_SECOND: f32 = 1000.0;

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
    let matches = App::new("ROS Node Tracker")
    .version("1.0")
    .author("Rony Gabbai")
    .about("Camera & object detection pipe")
    .arg(Arg::new("fps")
         .short('f')
         .long("fps")
         .value_name("FPS")
         .help("Sets the publish rate - fps")
         .takes_value(true)
         .required(false)
         .default_value("0.5")
         .validator(|v| v.parse::<f32>().map(|_| ()).map_err(|_| "FPS must be a float: between 0.0 - 1.0".to_string())))
    .arg(Arg::new("threshold")
         .short('t')
         .long("thr")
         .value_name("THR")
         .help("Sets the detection threshold. Defualt:0.4")
         .takes_value(true)
         .required(false)
         .default_value("0.4")
         .validator(|v| v.parse::<f32>().map(|_| ()).map_err(|_| "FPS must be a float: between 0.0 - 1.0".to_string())))
    .arg(Arg::new("model")
         .short('a')
         .long("model")
         .value_name("MODEL")
         .help("Sets the AI model: A,B (A:yolov8n_hen_bucket_cone_640 B:roktrack_yolov8_nano_fixed_640_640)")
         .takes_value(true)
         .required(false)
         .default_value("A")  
         .possible_values(&["A","B"])) 
    .arg(Arg::new("mode")
         .short('m')
         .long("mode")
         .value_name("MODE")
         .help("Sets the mode: None,low,med or high BW (None: no image,low:320x180 BW  med:320x180 color image, high:640x360 color)")
         .takes_value(true)
         .required(false)
         .default_value("high")  // Default FPS value
         .possible_values(&["none","low", "med", "high"]))     
    .arg(Arg::new("verbose")
         .short('v')
         .long("verbose")
         .help("Sets verbosity on")
         .takes_value(false)
         .required(false))
    .get_matches();



    let fps = matches.value_of("fps").unwrap().parse::<f32>().unwrap();
    let mode = matches.value_of("mode").unwrap().to_string();
    let model = matches.value_of("model").unwrap().to_string();
    let thr = matches.value_of("threshold").unwrap().parse::<f32>().unwrap();
    let verbose_mode = matches.is_present("verbose");


    println!("FPS: {}", fps);
    println!("Mode: {}", mode);
    println!("Model: {}", model);
    println!("Thr: {}",thr);
    println!("Verbose mode is {}", if verbose_mode { "on" } else { "off" });

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


    let period_ms: u64 = (MILLISECONDS_PER_SECOND / fps).round() as u64;
    println!(">FPS:{fps} period [ms]:{period_ms}");
    match mode.as_str() {
        "none" => {
            println!("No debug image ");
        },
        "low" => {
            println!("Black & white image 320x180");
        },
        "med" => {
            println!("Color image 320x180");
        },
        "high" => {
            println!("Color image 640x360");
        },
        _ => unreachable!("Mode should be either 'none', 'med', low' or 'high'"), // This case should never happen 
    }



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
        // Load the image from the captured data
        let mut img = match image::load_from_memory(&image_data) {
            Ok(img) => img,
            Err(e) => {
                eprintln!("Failed to load image from memory: {}", e);
                return; // Decide how to handle the error
            }
        };

        // Resize the image to smaller size to save BW
        let mut disble_image_publisher = false;
        let mut image_x = 640;
        let mut image_y = 360;

        match mode.as_str() {
            "none" => {
                disble_image_publisher = true;
            },
            "low" => {
                image_x = 320;
                image_y = 180;
                img = DynamicImage::ImageLuma8(img.to_luma8()) // Convert the image to grayscale for "low" mode
            },
            "med" => {
                image_x = 320;
                image_y = 180;
            },
            "high" => {
            },
            _ => unreachable!("Mode should be either 'none', 'med', low' or 'high'"), // This case should never happen 
        }


        let resized_img = img.resize_exact(image_x, image_y, image::imageops::FilterType::Nearest);
        // Convert the resized image back to a byte vector
        let mut resized_data = Vec::new();
        let mut cursor = Cursor::new(&mut resized_data);
        match resized_img.write_to(&mut cursor, ImageOutputFormat::Jpeg(80)) { // use quality 80 - TODO check if can decrease/increase
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to write resized image to buffer: {}", e);
                return; // Decide how to handle the error
            }
        }

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
            //data: image_data,          //TODO keep for orig mode.
            data: resized_data,

        };

        // Publish the image
        if !disble_image_publisher {
            match image_publisher.publish(&image_message) {
                Ok(_) =>  {},//rclrust_info!(logger, "Image published successfully."),
                Err(e) => eprintln!("Failed to publish image: {}", e),
            };
        };

        // Detect stage
        //println!("Detection starts!");
        let detect_res = obj_detect::detect("image.jpg",verbose_mode,&model,thr);
        //process string to DetObj format

        let mut detected_objects: Vec<DetObj> = Vec::new();
        // Estimate Pylon distance in cm 
        // Given data points


        for detection in &detect_res {
            let pixel_height:f64 = (detection.3 - detection.1).into(); 
            let obj = DetObj {
                box_location: BoxCor(detection.0, detection.1, detection.2, detection.3),
                otype: detection.4.to_string(),
                prob: detection.5,
                dist: estimation::estimate_distance(pixel_height),
            };
            //println!("Object:{:?} Pixel hieght:{}",obj.otype,pixel_height);
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
        if verbose_mode {
            rclrust_info!(logger, "Publishing: '{}'", message.data);
        }
        publisher.publish(&message).unwrap();
    })?;

    node.wait();

    Ok(())
}
