# SPAYD (Short Payment Descriptor)

This library implements text processing for the Short Payment Descriptor format 
(SPAYD or SPD). This is a simple text format for payment requests, providing all
the details required to make a bank transfer to the payee (destination account (IBAN),
amount etc). The data is usually displayed as a QR code that canc be scanned into
a banking app. Although it is mainly used in Czechia and the Slovakia, the
format is designed to handle international bank accounts and currencies.

This library only aims to handle the text processing of the data.
QR scanning is already implemented in libraries such as
[qr_code](https://crates.io/crates/qr_code) and may be provided by other
methods on mobile OSs.
