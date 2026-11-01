use comfy_table::Table;

use crate::http_client::HttpMethods;

pub struct TableInfo {
    target: String,
    method: HttpMethods,
    payload: String,
    conditional: String,
    rps: u64,
    concurrent: usize,
}

impl TableInfo {
    pub fn new(
        target: &str,
        method: HttpMethods,
        payload: &str,
        conditional: &str,
        rps: u64,
        concurrent: usize,
    ) -> Self {
        let target = String::from(target);
        let payload = String::from(payload);
        let conditional = String::from(conditional);
        TableInfo {
            target,
            method,
            payload,
            conditional,
            rps,
            concurrent,
        }
    }

    pub fn print_info(self) {
        let mut table = Table::new();
        table.add_row(vec!["Target:", self.target.as_str()]);
        table.add_row(vec!["Method:", self.method.to_string().as_str()]);
        table.add_row(vec!["Payload:", self.payload.as_str()]);
        table.add_row(vec!["Conditional:", self.conditional.as_str()]);
        table.add_row(vec!["Request per second:", self.rps.to_string().as_str()]);
        table.add_row(vec!["Concurrent:", self.concurrent.to_string().as_str()]);
        println!("{table}");
    }
}
