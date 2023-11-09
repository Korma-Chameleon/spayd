/// Main account number for the payment, may be an IBAN or a combined IBAN and BIC.
pub const ACCOUNT: &str = "ACC";
/// Alternative accounts that may be used instead of the main account. Often used
/// to find a destination bank where the transfer fees will be lower.
pub const ALTERNATIVE_ACCOUNTS: &str = "ALT-ACC";
/// Payment amount to send.
pub const AMOUNT: &str = "AM";
/// Payment currency.
pub const CURRENCY: &str = "CC";
/// A reference numer for the payee.
pub const REFERENCE: &str = "RF";
/// Payee's name.
pub const RECIPIENT: &str = "RN";
/// Date when the payment is due.
pub const DUE_DATE: &str = "DT";
/// Type of payment.
pub const PAYMENT_TYPE: &str = "PT";
/// A message for the payee to help identify the payment.
pub const MESSAGE: &str = "MSG";
/// CRC32 checksum for integrity verification.
pub const CRC32_CHECKSUM: &str = "CRC32";
