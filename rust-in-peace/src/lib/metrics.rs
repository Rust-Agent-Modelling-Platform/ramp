use hyper::{header::CONTENT_TYPE, rt::Future, service::service_fn_ok, Body, Response, Server};
use prometheus::{Encoder, GaugeVec, IntGaugeVec, TextEncoder};
use std::collections::HashMap;
use std::net::SocketAddr;

lazy_static! {
    static ref TOTAL_RECV_MESSAGES_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "messages_recv_total",
        "total messages received from source by target from last request",
        &["source", "target", "status"]
    )
    .unwrap();
}

pub type MetricName = String;

pub struct MetricHub {
    pub int_gauges_vec: HashMap<MetricName, IntGaugeVec>,
    pub gauges_vec: HashMap<MetricName, GaugeVec>,
}

impl MetricHub {
    pub fn register_gauge_vec(&mut self, name: &str, desc: &str, labels: &[&str]) {
        self.gauges_vec.insert(
            name.to_owned(),
            register_gauge_vec!(name, desc, labels).unwrap(),
        );
    }

    pub fn set_gauge_vec(&self, name: &str, labels: &[&str], value: f64) {
        if let Some(gauge) = self.gauges_vec.get(name) {
            gauge.with_label_values(labels).set(value);
        }
    }
    
    pub fn add_gauge_vec(&self, name: &str, labels: &[&str], value: f64) {
        if let Some(gauge) = self.gauges_vec.get(name) {
            gauge.with_label_values(labels).add(value);
        }
    }

    pub fn inc_gauge_vec(&self, name: &str, labels: &[&str]) {
        if let Some(gauge) = self.gauges_vec.get(name) {
            gauge.with_label_values(labels).inc();
        }
    }

    pub fn reset_gauge_vec(&self, name: &str) {
        if let Some(gauge) = self.gauges_vec.get(name) {
            gauge.reset();
        }
    }

    pub fn register_int_gauge_vec(&mut self, name: &str, desc: &str, labels: &[&str]) {
        self.int_gauges_vec.insert(
            name.to_owned(),
            register_int_gauge_vec!(name, desc, labels).unwrap(),
        );
    }

    pub fn set_int_gauge_vec(&self, name: &str, labels: &[&str], value: i64) {
        if let Some(gauge) = self.int_gauges_vec.get(name) {
            gauge.with_label_values(labels).set(value);
        }
    }

    pub fn add_int_gauge_vec(&self, name: &str, labels: &[&str], value: i64) {
        if let Some(gauge) = self.int_gauges_vec.get(name) {
            gauge.with_label_values(labels).add(value);
        }
    }

    pub fn inc_int_gauge_vec(&self, name: &str, labels: &[&str]) {
        if let Some(gauge) = self.int_gauges_vec.get(name) {
            gauge.with_label_values(labels).inc();
        }
    }

    pub fn reset_int_gauge_vec(&self, name: &str) {
        if let Some(gauge) = self.int_gauges_vec.get(name) {
            gauge.reset();
        }
    }
}

impl Default for MetricHub {
    fn default() -> Self {
        Self {
            int_gauges_vec: HashMap::new(),
            gauges_vec: HashMap::new(),
        }
    }
}

pub fn start_server(address: String) {
    let addr: SocketAddr = address.parse().unwrap();
    let new_service = || {
        let encoder = TextEncoder::new();
        service_fn_ok(move |_request| {
            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            encoder.encode(&metric_families, &mut buffer).unwrap();
            let response = Response::builder()
                .status(200)
                .header(CONTENT_TYPE, encoder.format_type())
                .body(Body::from(buffer))
                .unwrap();
            TOTAL_RECV_MESSAGES_GAUGE.reset();
            response
        })
    };

    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| log::error!("Server error: {}", e));

    log::info!("Metrics are exposed under: {:?}", addr);
    hyper::rt::run(server);
}

pub fn inc_received_messages(from: String, target: String, status: String) {
    TOTAL_RECV_MESSAGES_GAUGE
        .with_label_values(&[from.as_str(), target.as_str(), status.as_str()])
        .inc();
}
