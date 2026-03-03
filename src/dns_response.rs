pub struct DnsResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
    pub errors: Option<Vec<String>>,
}
