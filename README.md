# SPAYD (Short Payment Descriptor)

SPAYD (or SPD) is a simple text format for storing and sharing payment
information used in the Czech Republic and Slovakia. It can encode details of
the payee, destination account (IBAN), amount etc. Typically SPAYDs are sent as
QR codes which can be scanned into a mobile banking app.

## TODO
Output SPAYD structure as text
Enforce Required fields
Percent encode/decode of non-ascii
CRC32 checks
Value convewrsions for dates, IBAN, currency symbol etc
