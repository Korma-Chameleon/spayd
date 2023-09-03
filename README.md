# SPAYD (Short Payment Descriptor)

This library implements text processing for the Short Payment Descriptor format 
(SPAYD or SPD). This is a simple text format for requesting payments, mainly
used in Czechia and the Slovak Republic. It can encode details of the payee,
destination account (IBAN), amount etc.

While Typically SPAYDs are sent as QR codes which can be scanned into a mobile
banking app, this library only aims to handle the text processing of the data.
QR scanning is already implemented in libraries such as
[qr_code](https://crates.io/crates/qr_code) and may be provided by other
methods on mobile OSs.

## TODO
- Enforce Required fields
- Value conversions for dates, IBAN, currency symbol etc
- Add Cargo keywords
