use open_meteo_api::models::TimeZone;
use open_meteo_api::query::OpenMeteo;
use serde_json::Value;
use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn get_city_temperature_by_name(c_city_name: *const c_char) -> f32 {
    // for NULL-terminated C strings, it's a little bit clumsier
    let city_name = unsafe { CStr::from_ptr(c_city_name).to_string_lossy().into_owned() };

    println!("city_name = <{:?}>", city_name);

    // call get_weather_data(city_name) here
    //TBD: this is not acceptable as it will block the main thread to be check later
    let city: &str = city_name.as_str();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut temp: f32 = 0.0;
    rt.block_on(async {
        //call get_weather_data_for_city() and print the returned temperature
        let result = get_weather_data_for_city(city).await;
        temp = result.unwrap();
    });
    return temp;
}

#[no_mangle]
pub extern "C" fn get_city_temperature_by_geometry(lat: f32, lng: f32) -> f32 {
    println!("Latitude = <{}> Longtude = {}", lat, lng);

    //TBD: this is not acceptable as it will block the main thread to be check later
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut temp: f32 = 0.0;
    rt.block_on(async {
        //call get_weather_data_for_geocode() and print the returned temperature
        let result = get_weather_data_for_geocode(lat, lng).await;
        temp = result.unwrap();
    });
    return temp;
}

#[no_mangle]
pub extern "C" fn is_city_name_valid(c_city_name: *const c_char) -> i32 {
    // for NULL-terminated C strings, it's a little bit clumsier
    let city_name = unsafe { CStr::from_ptr(c_city_name).to_string_lossy().into_owned() };

    let city: &str = city_name.as_str();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut ret = 0;
    rt.block_on(async {
        //call get_geocode_from_cityname() to check if the city is valid
        let result = get_geocode_from_cityname(city).await;
        // check if the city is valid by checking the coordinates
        if result.is_err() {
            println!("Invalid City Name!");
            ret = -1;
        } else {
            println!("Valid City Name!");
        }
    });
    return ret;
}

#[no_mangle]
pub extern "C" fn is_city_geometry_valid(lat: f32, lng: f32) -> i32 {
    /*The valid range of latitude in degrees is -90 and +90
    for the southern and northern hemisphere, respectively.
    Longitude is in the range -180 and +180 specifying coordinates
    west and east of the Prime Meridian, respectively*/

    if lat < -90.0 || lat > 90.0 {
        println!("Invalid Latitude");
        return -2;
    }

    if lng < -180.0 || lng > 180.0 {
        println!("Invalid Longitude");
        return -3;
    }

    return 0;
}

async fn get_geocode_from_cityname(city: &str) -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let url = format!(
        "https://geocode.maps.co/search?q={}&format=json&api_key=665ca3fbc910d600112365lrt1a225b",
        city
    );

    let response = reqwest::get(url).await?.text().await?;

    //check if response is empty or contains valid data and print the response
    match response.as_str() {
        "[]" => {
            println!("Empty response: Not a valid City Name!");
            //return invalid coordinates
            return Err("Invalid City Name!".into());
        }
        _ => {
            println!("Valid City Name!");
        }
    }

    println!("Response: {}", response);

    // parsed json with (almost) all data you may need
    // for more info see open-meteo.com/en/docs
    let _json: Value = serde_json::from_str(&response).expect("test .coordinates() instead".into());

    let (lat, lon) = (
        _json[0]["lat"].as_str().unwrap().parse::<f32>().unwrap(),
        _json[0]["lon"].as_str().unwrap().parse::<f32>().unwrap(),
    );

    Ok((lat, lon))
}

async fn get_weather_data_for_city(city: &str) -> Result<f32, Box<dyn std::error::Error>> {
    let result = get_geocode_from_cityname(city).await;
    // check if the city is valid by checking the coordinates
    if result.is_err() {
        println!("Invalid City Name!");
        return Err("Invalid City Name!".into());
    }

    let (lat, lon) = result.unwrap();

    let data1 = OpenMeteo::new()
        .coordinates(lat, lon)? // add location
        .current_weather()? // add current weather data
        .time_zone(TimeZone::EuropeLondon)? // set time zone for using .daily()
        .hourly()? // add hourly weather data
        .query()
        .await?;

    // check if the data is available
    if data1.current_weather.is_none() {
        println!("No data available for this location");
        return Err("No data available for this location".into());
    }

    let temperature = data1.current_weather.unwrap().temperature;

    println!("{}", temperature);

    Ok(temperature)
}

async fn get_weather_data_for_geocode(lat: f32, lon: f32) -> Result<f32, Box<dyn std::error::Error>> {
    // check if the city is valid by checking the coordinates

    let data1 = OpenMeteo::new()
        .coordinates(lat, lon)? // add location
        .current_weather()? // add current weather data
        .time_zone(TimeZone::EuropeLondon)? // set time zone for using .daily()
        .hourly()? // add hourly weather data
        .query()
        .await?;

    // check if the data is available
    if data1.current_weather.is_none() {
        println!("No data available for this location");
        return Err("No data available for this location".into());
    }

    let temperature = data1.current_weather.unwrap().temperature;

    println!("{}", temperature);

    Ok(temperature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_geocode_from_cityname_valid_city() {
        let city = "Egypt";
        let result = get_geocode_from_cityname(city).await;
        assert!(result.is_ok());
        let (lat, lon) = result.unwrap();
        assert_eq!(lat, 26.2540493);
        assert_eq!(lon, 29.2675469);
    }

    #[tokio::test]
    async fn test_get_geocode_from_cityname_invalid_city() {
        let city = "InvalidCity";
        let result = get_geocode_from_cityname(city).await;
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert_eq!(error_message, "Invalid City Name!");
    }

    #[tokio::test]
    async fn test_get_weather_data_for_city_valid_city() {
        let city = "London";
        let result = get_weather_data_for_city(city).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_weather_data_for_city_invalid_city() {
        let city = "InvalidCity";
        let result = get_weather_data_for_city(city).await;
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert_eq!(error_message, "Invalid City Name!");
    }

    #[tokio::test]
    async fn test_get_weather_data_for_geocode_valid_geocode() {
        let lat = 51.5074;
        let lon = -0.1278;
        let result = get_weather_data_for_geocode(lat, lon).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_weather_data_for_geocode_invalid_geocode() {
        let lat = 100.0;
        let lon = 200.0;
        let result = get_weather_data_for_geocode(lat, lon).await;
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert_eq!(error_message, "Latitude must be in range of -90 to 90°. Given: 100.0.");
    }
}
