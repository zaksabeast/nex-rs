use no_std_io::{StreamReader, StreamWriter, Reader};
use crate::stream::{StreamIn, StreamOut};

pub trait StructureInterface {
    //fn hierarchy(&self) -> Vec<Box<dyn StructureInterface>> {
    //    Vec::new()
    //}

    fn extract_from_stream<T: Reader>(&mut self, stream: &mut StreamIn<T>) -> core::result::Result<(), &'static str> {
        Ok(())
    }

    fn bytes(&mut self, stream: StreamOut) -> Vec<u8> {
        Vec::new()
    }
}

pub struct NullData;

impl NullData {
    pub fn new() -> Self {
        Self{}
    }
}

impl StructureInterface for NullData {}

pub struct RVConnectionData {
    station_url: String,
    special_protocols: Vec<u8>,
    station_url_special_protocols: String,
    time: u64
}

impl RVConnectionData {
    pub fn new() -> Self {
        Self {
            station_url: String::new(),
            special_protocols: Vec::new(),
            station_url_special_protocols: String::new(),
            time: 0
        }
    }

    pub fn set_station_url(&mut self, station_url: String) {
        self.station_url = station_url;
    }

    pub fn set_special_protocols(&mut self, special_protocols: Vec<u8>) {
        self.special_protocols = special_protocols;
    }

    pub fn set_station_url_special_protocols(&mut self, station_url_special_protocols: String) {
        self.station_url_special_protocols = station_url_special_protocols;
    }

    pub fn set_time(&mut self, time: u64) {
        self.time = time;
    }
}

impl StructureInterface for RVConnectionData {
    fn bytes(&mut self, mut stream: StreamOut) -> Vec<u8> {
        stream.write_string(&self.station_url);
        stream.checked_write_stream_le(&0_u32);
        stream.write_string(&self.station_url_special_protocols);
        stream.checked_write_stream_le(&self.time);

        stream.into()
    }
}

pub struct DateTime {
    value: u64
}

impl DateTime {
    pub fn new_from_value(value: u64) -> Self {
        Self {
            value
        }
    }

    pub fn make(&mut self, year: u64, month: u64, day: u64, hour: u64, minute: u64, second: u64) -> u64 {
        self.value = second | (minute << 6) | (hour << 12) | (day << 17) | (month << 22) | (year << 26);
        self.value
    }

    pub fn get_value(&self) -> u64 {
        self.value
    }
}

pub struct StationURL {
    scheme: Option<String>,
    address: Option<String>,
    port: Option<String>,
    stream: Option<String>,
    sid: Option<String>,
    cid: Option<String>,
    pid: Option<String>,
    transport_type: Option<String>,
    rvcid: Option<String>,
    natm: Option<String>,
    natf: Option<String>,
    upnp: Option<String>,
    pmp: Option<String>,
    probe_init: Option<String>,
    prid: Option<String>
}

impl Default for StationURL {
    fn default() -> Self {
        Self {
            scheme: None,
            address: None,
            port: None,
            stream: None,
            sid: None,
            cid: None,
            pid: None,
            transport_type: None,
            rvcid: None,
            natm: None,
            natf: None,
            upnp: None,
            pmp: None,
            probe_init: None,
            prid: None
        }
    }
}

impl StationURL {
    pub fn set_scheme(&mut self, scheme: String) {
        self.scheme = Some(scheme);
    }

    pub fn set_address(&mut self, address: String) {
        self.address = Some(address);
    }

    pub fn set_port(&mut self, port: String) {
        self.port = Some(port);
    }

    pub fn set_stream(&mut self, stream: String) {
        self.stream = Some(stream);
    }

    pub fn set_sid(&mut self, sid: String) {
        self.sid = Some(sid);
    }

    pub fn set_cid(&mut self, cid: String) {
        self.cid = Some(cid);
    }

    pub fn set_pid(&mut self, pid: String) {
        self.pid = Some(pid);
    }

    pub fn set_transport_type(&mut self, transport_type: String) {
        self.transport_type = Some(transport_type);
    }

    pub fn set_rvcid(&mut self, rvcid: String) {
        self.rvcid = Some(rvcid);
    }

    pub fn set_natm(&mut self, natm: String) {
        self.natm = Some(natm);
    }

    pub fn set_natf(&mut self, natf: String) {
        self.natf = Some(natf);
    }

    pub fn set_upnp(&mut self, upnp: String) {
        self.upnp = Some(upnp);
    }

    pub fn set_pmp(&mut self, pmp: String) {
        self.pmp = Some(pmp);
    }

    pub fn set_probe_init(&mut self, probe_init: String) {
        self.probe_init = Some(probe_init);
    }

    pub fn set_prid(&mut self, prid: String) {
        self.prid = Some(prid);
    }

    pub fn get_scheme(self) -> Option<String> {
        self.scheme
    }

    pub fn get_address(self) -> Option<String> {
        self.address
    }

    pub fn get_port(self) -> Option<String> {
        self.port
    }

    pub fn get_stream(self) -> Option<String> {
        self.stream
    }

    pub fn get_sid(self) -> Option<String> {
        self.sid
    }

    pub fn get_cid(self) -> Option<String> {
        self.cid
    }

    pub fn get_pid(self) -> Option<String> {
        self.pid
    }

    pub fn get_transport_type(self) -> Option<String> {
        self.transport_type
    }

    pub fn get_rvcid(self) -> Option<String> {
        self.rvcid
    }

    pub fn get_natm(self) -> Option<String> {
        self.natm
    }

    pub fn get_natf(self) -> Option<String> {
        self.natf
    }

    pub fn get_upnp(self) -> Option<String> {
        self.upnp
    }

    pub fn get_pmp(self) -> Option<String> {
        self.pmp
    }

    pub fn get_probe_init(self) -> Option<String> {
        self.probe_init
    }

    pub fn get_prid(self) -> Option<String> {
        self.prid
    }

    pub fn new_from_string(str: String) -> Self {
        if str == "".to_string() {
            return Self::default();
        }

        let mut station_url = Self::default();
        let mut split = str.split(":/");

        station_url.scheme = Some(split.next().unwrap_or_else(|| "").to_string());
        let fields = split.next().unwrap_or_else(|| "");

        let mut params = fields.split(";");

        while let Some(param) = params.next() {
            let mut split = param.split("=");

            let name = split.next().unwrap_or_else(|| "");
            let value: Option<String> = Some(split.next().unwrap_or_else(|| "").to_string());
            match name {
                "address" => station_url.address = value,
                "port" => station_url.port = value,
                "stream" => station_url.stream = value,
                "sid" => station_url.sid = value,
                "CID" => station_url.cid = value,
                "PID" => station_url.pid = value,
                "type" => station_url.transport_type = value,
                "RVCID" => station_url.rvcid = value,
                "natm" => station_url.natm = value,
                "natf" => station_url.natf = value,
                "upnp" => station_url.upnp = value,
                "pmp" => station_url.pmp = value,
                "probeinit" => station_url.probe_init = value,
                "PRID" => station_url.prid = value,
                _ => {}
            }
        }
        station_url
    }

    pub fn encode_to_string(&self) -> String {
        let mut fields = Vec::new();

        if let Some(address) = &self.address {
            fields.push(address.to_string());
        }

        if let Some(port) = &self.port {
            fields.push(port.to_string());
        }

        if let Some(stream) = &self.stream {
            fields.push(stream.to_string());
        }

        if let Some(sid) = &self.sid {
            fields.push(sid.to_string());
        }

        if let Some(cid) = &self.cid {
            fields.push(cid.to_string());
        }

        if let Some(pid) = &self.pid {
            fields.push(pid.to_string());
        }

        if let Some(transport_type) = &self.transport_type {
            fields.push(transport_type.to_string());
        }

        if let Some(rvcid) = &self.rvcid {
            fields.push(rvcid.to_string());
        }

        if let Some(natm) = &self.natm {
            fields.push(natm.to_string());
        }

        if let Some(natf) = &self.natf {
            fields.push(natf.to_string());
        }

        if let Some(upnp) = &self.upnp {
            fields.push(upnp.to_string());
        }

        if let Some(pmp) = &self.pmp {
            fields.push(pmp.to_string());
        }

        if let Some(probe_init) = &self.probe_init {
            fields.push(probe_init.to_string());
        }

        if let Some(prid) = &self.prid {
            fields.push(prid.to_string());
        }

        let fields_string = fields.join(";");
        let scheme = if let Some(scheme) = &self.scheme {
            scheme.to_string()
        } else {
            "".to_string()
        };

        format!("{}:/{}", scheme, fields_string)
    }
}

struct Result {
    result_code: u32
}

impl Result {
    pub fn new_from_code(result_code: u32) -> Self {
        Self {
            result_code
        }
    }

    pub fn extract_from_stream<T: Reader>(&mut self, stream: &mut StreamIn<T>) -> core::result::Result<(), &'static str> {
        self.result_code = stream
            .read_stream_le::<u32>()
            .map_err(|_| "Result code could not be read")?
            .try_into()
            .map_err(|_|"Result code size does not fit into u32")?;

        Ok(())
    }

    pub fn bytes(&self, mut stream: StreamOut) -> Vec<u8> {
        stream.checked_write_stream_le(&self.result_code);
        stream.into()
    }
}

struct ResultRange {
    offset: u32,
    length: u32
}

impl ResultRange {
    pub fn new() -> Self {
        Self {
            offset: 0,
            length: 0
        }
    }
}

impl StructureInterface for ResultRange {
    fn extract_from_stream<T: Reader>(&mut self, stream: &mut StreamIn<T>) -> std::result::Result<(), &'static str> {
        self.offset = stream
            .read_stream_le::<u32>()
            .map_err(|_| "Offset could not be read")?
            .try_into()
            .map_err(|_|"Offset size does not fit into u32")?;

        self.length = stream
            .read_stream_le::<u32>()
            .map_err(|_| "Length could not be read")?
            .try_into()
            .map_err(|_|"Length size does not fit into u32")?;

        Ok(())
    }
}

struct DataHolder<T: StructureInterface> {
    name: String,
    object: T
}

impl <T : StructureInterface>DataHolder<T> {
    pub fn new_from_object(object: T) -> Self {
        Self {
            name: String::new(),
            object
        }
    }
}

impl<T: StructureInterface> StructureInterface for DataHolder<T> {
    fn bytes(&mut self, mut stream: StreamOut) -> Vec<u8> {
        let content = self.object.bytes(StreamOut::new());

        stream.write_string(&self.name);
        stream.checked_write_stream_le(&(content.len() as u32 + 4_u32));
        stream.write_buffer(&content);
        stream.into()
    }
}