#[derive(Default)]
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
    prid: Option<String>,
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
        if str == *"" {
            return Self::default();
        }

        let mut station_url = Self::default();
        let mut split = str.split(":/");

        station_url.scheme = Some(split.next().unwrap_or("").to_string());
        let fields = split.next().unwrap_or("");

        let params = fields.split(';');

        for param in params {
            let mut split = param.split('=');

            let name = split.next().unwrap_or("");
            let value: Option<String> = Some(split.next().unwrap_or("").to_string());
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
