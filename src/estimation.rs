// Distance estimation 
// Third-degree polynomial fit based on 7 points in Excel - TBD add more points in field experiment
// dist = A*h^3 + B*h^2 + C*h + D

const A: f64 = -5.86230652281417E-05;
const B: f64 =  0.041512419539938;
const C: f64 = -9.70395960666584;
const D: f64 =  877.331591326026;

//const A: f64 = -0.000176893151865;
//const B: f64 =  0.088553526574368;
//const C: f64 = -15.2187096519726;
//const D: f64 = 1067.41602184761;

const CM_IN_METER: f64 = 100.0;


// Function to estimate distance using linear regression parameters -return distnce in [Meter]
pub fn estimate_distance(pixel_height: f64, otype:&str) -> f64 {

    //normalize height to cone height 
    // TODO later on prepare constants per otype 
    let mut normalized_pixel_height :f64 = 0.0;
    match otype {
        "cone"      =>  {normalized_pixel_height = pixel_height;}
        "pylon"     =>  {normalized_pixel_height = pixel_height;}
        "bucket"    =>  {normalized_pixel_height = pixel_height*2.3;}
        "hen"       =>  {normalized_pixel_height = pixel_height*2.0;}
        "nothing"   =>  {normalized_pixel_height = 0.0;}
        _ => unreachable!("type not recognaized"), // This case should never happen 
    }

    //let dist=((A*pixel_height.powi(3)+B*pixel_height.powi(2)+C*pixel_height + D) / CM_IN_METER * 100.0).round() / 100.0;
    let dist=((A*normalized_pixel_height.powi(3)+B*normalized_pixel_height.powi(2)+C*normalized_pixel_height + D) / CM_IN_METER * 100.0).round() / 100.0;
    dist
}