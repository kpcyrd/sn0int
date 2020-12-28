use crate::errors::*;

use sloppy_rfc4880::{self, Tag, Signature};
use crate::engine::ctx::State;
use crate::engine::structs::{LuaMap, LuaList, byte_array};
use crate::hlua::{self, AnyLuaValue};
use std::collections::HashSet;
use std::sync::Arc;
use std::io::BufReader;


fn is_new_signing_key(seen: &mut HashSet<String>, sig: &Signature) -> bool {
    let key = if let Some(fp) = &sig.fingerprint {
        fp
    } else if let Some(keyid) = &sig.keyid {
        keyid
    } else {
        return false;
    };

    if seen.contains(key) {
        return false;
    }

    seen.insert(key.to_string());
    true
}

fn pgp_pubkey_lua(pubkey: &[u8]) -> Result<AnyLuaValue> {
    let mut fingerprint = None;
    let mut uids = LuaList::new();
    let mut sigs = LuaList::new();

    let mut signing_keys_seen = HashSet::new();

    for (tag, body) in sloppy_rfc4880::Parser::new(pubkey) {
        match tag {
            Tag::PublicKey => {
                let fp = sloppy_rfc4880::pubkey::fingerprint(&body);
                fingerprint = Some(fp);
            },
            Tag::UserID => {
                if let Ok(body) = String::from_utf8(body) {
                    uids.push_str(body);
                }
            },
            Tag::Signature => {
                if let Ok(sig) = sloppy_rfc4880::signature::parse(&body) {
                    if is_new_signing_key(&mut signing_keys_seen, &sig) {
                        sigs.push_serde(sig)?;
                    }
                }
            },
            _ => (),
        }
    }

    let mut map = LuaMap::new();
    if let Some(fp) = fingerprint {
        map.insert_str("fingerprint", fp);
    }
    if !uids.is_empty() {
        map.insert("uids", uids);
    }
    if !sigs.is_empty() {
        map.insert("sigs", sigs);
    }
    Ok(map.into())
}

pub fn pgp_pubkey(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("pgp_pubkey", hlua::function1(move |pubkey: AnyLuaValue| -> Result<AnyLuaValue> {
        let pubkey = byte_array(pubkey)
            .map_err(|err| state.set_error(err))?;
        pgp_pubkey_lua(&pubkey)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn pgp_pubkey_armored(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("pgp_pubkey_armored", hlua::function1(move |pubkey: String| -> Result<AnyLuaValue> {
        let mut r = BufReader::new(pubkey.as_bytes());
        let pubkey = sloppy_rfc4880::armor::read_armored(&mut r)
            .map_err(|err| state.set_error(format_err!("{:#}", err)))?;

        pgp_pubkey_lua(&pubkey)
            .map_err(|err| state.set_error(err))
    }))
}


#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    fn verify_pgp_pubkey_armored() {
        let script = Script::load_unchecked(r#"
        function run()
            key = pgp_pubkey_armored([===[
-----BEGIN PGP PUBLIC KEY BLOCK-----
Version: GnuPG v2
Foo: bar

mQENBFu6q90BCADgD7Q9aH5683yt7hzPktDkAUNAZJHwYhUNeyGK43frPyDRWQmq
N+oXTfiYWLQN+d7KNBTnF9uwyBdaLM7SH44lLNYo8W09mVM2eK+wt19uf5HYNgAE
8la45QLo/ce9CQVe1a4oXNWq6l0FOY7M+wLe+G2wMwz8RXGgwd/qQp4/PB5YpUhx
nAnzClxvwymrL6BQXsRcKSMSD5bIzIv95n105CvW5Hql7JR9zgOR+gHqVOH8HBUc
ZxMumrTM6aKLgAhgM8Sn36gCFOfjlG1b1OFLZhUtgro/nnEOmAurRsCZy8M5h8QM
FpZChIH8kgHs90F/CCvGjMq3qvWcH8ZsPUizABEBAAG0NUhhbnMgQWNrZXIgKGV4
YW1wbGUgY29tbWVudCkgPGhhbnMuYWNrZXJAZXhhbXBsZS5jb20+iQFOBBMBCAA4
FiEEyzeO1eEwbB03hcqBM00IodGdlj8FAlu6q90CGwMFCwkIBwIGFQgJCgsCBBYC
AwECHgECF4AACgkQM00IodGdlj/AJQgAjmk+iP5b7Jt7+f+lU4Oprlf3f3DG/uh5
Ge6MjV7cvtxlhZJRD5hxGt9RwwnEp61TBSbrem288pM89ilQfTNe0wUr9OzwWzh/
8Ngl5iWnD2ah3Mpi5R1V/YMNf2cnwVjqNvfkRHdNc43pZOkC2GoiTUn0QY0UBpOW
ZMN3//ANi6ZtiK/L0IZQND/gKvOzu/4tfaJeBl26T3cVYj53p3G3jhlb92vVa8SR
uL3S3bzd1h5snDgU1uXHmNHGbhkEc4KUneQ0V9/bdZrg6OzFAfM1ghgfoId+YpQH
er9L26ISL3QF58wdEXfIdHYEmMlANjBMO2cUlQXgONuCgkMuY7GBmrkBDQRbuqvd
AQgA41jqCumCxYV0NdSYNnTSSDRyd69dOUYCAPT80iZ739s7KKJS9X9KVfGmDjfi
u2RcfR/KYj53HoyOm4Pm/+ONN8De4ktzXpIpJxGC+O8NBvd9vkboAS6qnCjK7KVE
r91ymxxVKp2dzZvVfpIjWVZR5i2EAvS5vw8UK4gL8ALH+S9leJFZrQWcgyoJOJzH
Rzr9pesX2HvdgcNG1O6QUArlsnsTnqpi/hu7tQa8tifBpWDeArOA23Y2DgeehdDF
lSU/8KD4J+AkFrWWlcTaMsvSChXQkCHEMRIcSOfXtdpX5KJSE7UBQdD1opm+mR79
VeHnuJAAVZZtUZmJA7pjdKykYQARAQABiQE2BBgBCAAgFiEEyzeO1eEwbB03hcqB
M00IodGdlj8FAlu6q90CGwwACgkQM00IodGdlj8bMAf+Lq3Qive4vcrCTT4IgvVj
arOACdcbtt5RhVBTimT19rDWNH+m+PfPjo3FSlBj5cm70KAXUS2LBFFxhakTZ/Mq
cQroWZpVbBxj4kipEVVJZFdUZQaDERJql0xYGOQrNMQ4JGqJ84BRrtOExjSqo41K
hAhNe+bwPGH9/Igiixc4tH07xa7TOy4MyJv/6gpbHy/lW1hqpCAgM5fT/im5/6QF
k0tED6vIuc54IWiOmwCnjZiQnJ8uCwEu+cuJ5Exwy9CNERLp5v0y4eG+0E+at9j/
macOg39qf09t53pTqe9dWv5NIi319TeBsKZ2lb0crrQjsbHqk0DAUwgQuoANqLku
vA==
=kRIv
-----END PGP PUBLIC KEY BLOCK-----
]===])
            if last_err() then return end

            print(key)

            fingerprint = key['fingerprint']
            if fingerprint ~= 'CB378ED5E1306C1D3785CA81334D08A1D19D963F' then
                return 'Unexpected fingerprint: ' .. fingerprint
            end

            if key['uids'][1] ~= 'Hans Acker (example comment) <hans.acker@example.com>' then
                return 'Unexpected uid: ' .. key['uid']
            end

            keyid = key['sigs'][1]['keyid']
            if keyid ~= '334D08A1D19D963F' then
                return 'Unexpected keyid: ' .. keyid
            end

            sigfp = key['sigs'][1]['fingerprint']
            if sigfp ~= 'CB378ED5E1306C1D3785CA81334D08A1D19D963F' then
                return 'Unexpected sigfp: ' .. sigfp
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    #[ignore]
    fn verify_pgp_fetch_wkd() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            url = 'https://openpgpkey.archlinux.org/.well-known/openpgpkey/archlinux.org/hu/in9mwr4s84x7gm51851h343n3at1x61g?l=anthraxx'
            req = http_request(session, 'GET', url, {
                binary=true,
            })
            r = http_fetch(req)
            k = pgp_pubkey(r['binary'])
            info(k)
            if k['fingerprint'] ~= 'E240B57E2C4630BA768E2F26FC1B547C8D8172C8' then
                return 'wrong fingerprint: ' .. k['fingerprint']
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
