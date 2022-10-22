#[macro_use]
extern crate mime;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::str::FromStr;
use urlencoded::UrlEncodedBody;

/// Get greatest common divisor of two numbers
///
/// # Examples
/// Common divisor of 2 and 4 is 2
/// ```
/// assert_eq!(gcd(2, 4), 2);
/// ```
///
/// # Panics
/// Panics if any of the arguments is zero
/// ```
/// gcd(0, 1);
/// ```
fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }
        m = m % n;
    }
    n
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(14, 15), 1);

    assert_eq!(gcd(2 * 3 * 5 * 11 * 17, 3 * 7 * 11 * 13 * 19), 3 * 11);
}

/// Main function
fn main() {
    let mut router = Router::new();

    router.get("/", get_form, "index");
    router.post("/gcd", post_gcd, "post_gcd");

    println!("Server running on port 3000: http://localhost:3000/");
    Iron::new(router).http("localhost:3000").unwrap();
}

fn get_form(_request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();
    response.set_mut(status::Ok);
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    response.set_mut(
        r#"
        <title>GCD Calculator</title>
        <form action="/gcd" method="post">
            <input type="text" name="n"/>
            <input type="text" name="m"/>
            <button type="submit">Compute GCD</button>
        </form>
    "#,
    );
    Ok(response)
}

fn post_gcd(request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();

    let form_data = match request.get_ref::<UrlEncodedBody>() {
        Ok(map) => map,
        Err(e) => {
            response.set_mut(status::BadRequest);
            response.set_mut(format!("Error parsing form data: {:?}\n", e));
            return Ok(response);
        }
    };
    let n = match form_data.get("n") {
        Some(nums) => nums,
        None => {
            response.set_mut(status::BadRequest);
            response.set_mut(format!("form data has no 'n' parameter\n"));
            return Ok(response);
        }
    };
    let m = match form_data.get("m") {
        Some(nums) => nums,
        None => {
            response.set_mut(status::BadRequest);
            response.set_mut(format!("form data has no 'm' parameter\n"));
            return Ok(response);
        }
    };
    let mut numbers = Vec::new();
    for unparsed in n {
        match u64::from_str(unparsed) {
            Ok(n) => numbers.push(n),
            Err(_) => {
                response.set_mut(status::BadRequest);
                response.set_mut(format!(
                    "Value for 'n' parameter not a number: {:?}\n",
                    unparsed
                ));
                return Ok(response);
            }
        }
    }
    for unparsed in m {
        match u64::from_str(unparsed) {
            Ok(m) => numbers.push(m),
            Err(_) => {
                response.set_mut(status::BadRequest);
                response.set_mut(format!(
                    "Value for 'n' parameter not a number: {:?}\n",
                    unparsed
                ));
                return Ok(response);
            }
        }
    }

    let mut d = numbers[0];
    for m in &numbers[1..] {
        d = gcd(d, *m);
    }

    response.set_mut(status::Ok);
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    response.set_mut(format!(
        "The greatest common divisor of the numbers {:?} is <b>{}</b>\n",
        numbers, d
    ));

    Ok(response)
}
