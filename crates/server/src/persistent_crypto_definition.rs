//! Current persistent envelope domain owned by runtime and material export.
use xcss::secret_envelope::EnvelopeDomain;

pub(crate) struct ClientAuthorizationEnvelope;

impl EnvelopeDomain for ClientAuthorizationEnvelope {
    const DOMAIN: &'static [u8] = b"xsos/client-instance-authorization";
    const REVISION: u16 = 1;
}
