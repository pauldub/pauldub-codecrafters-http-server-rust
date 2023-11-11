use nom::{
    bytes::complete::{tag, take_till1},
    character::complete::{line_ending, space1},
    IResult,
};

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub http_version: String,
}

impl Request {
    pub fn from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (input, method) = tag("GET")(bytes)?;
        let (input, _) = space1(input)?;
        let (input, path) = take_till1(|c| c == b' ')(input)?;
        let (input, _) = space1(input)?;
        let (input, http_version) = tag("HTTP/1.1")(input)?;
        let (input, _) = line_ending(input)?;

        let request = Request {
            method: String::from_utf8_lossy(method).to_string(),
            path: String::from_utf8_lossy(path).to_string(),
            http_version: String::from_utf8_lossy(http_version).to_string(),
        };

        Ok((input, request))
    }
}

#[derive(Debug)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    pub fn from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (input, name) = take_till1(|c| c == b':')(bytes)?;
        let (input, _) = tag(": ")(input)?;
        let (input, value) = take_till1(|c| c == b'\r')(input)?;
        let (input, _) = line_ending(input)?;

        let header = Header {
            name: String::from_utf8_lossy(name).to_string(),
            value: String::from_utf8_lossy(value).to_string(),
        };

        Ok((input, header))
    }

    pub fn parse_all(bytes: &[u8]) -> IResult<&[u8], Vec<Self>> {
        let mut headers = Vec::new();
        let mut input = bytes;

        loop {
            // Stop when there are two lines separating the headers from the body
            if input.len() >= 4 && &input[0..4] == b"\r\n\r\n" {
                input = &input[4..];
                break;
            }

            if input.len() == 0 {
                break;
            }

            if input.len() == 2 && &input[0..2] == b"\r\n" {
                break;
            }

            let (leftover, header) = Header::from_bytes(input)?;
            input = leftover;
            headers.push(header);
        }

        Ok((input, headers))
    }
}
