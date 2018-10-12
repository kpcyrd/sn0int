use errors::*;

use sloppy_rfc4880::{self, Tag};
use engine::ctx::State;
use engine::structs::{LuaMap, LuaList, byte_array};
use hlua::{self, AnyLuaValue};
use std::sync::Arc;
use std::io::BufReader;


fn pgp_pubkey_lua(pubkey: Vec<u8>) -> Result<AnyLuaValue> {
    let mut uids = LuaList::new();

    for (tag, body) in sloppy_rfc4880::Parser::new(pubkey.as_slice()) {
        match tag {
            Tag::UserID => {
                let body = String::from_utf8(body)?;
                uids.push_str(body);
            },
            _ => (),
        }
    }

    let mut map = LuaMap::new();
    if !uids.is_empty() {
        map.insert("uids", uids);
    }
    Ok(map.into())
}

pub fn pgp_pubkey(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("pgp_pubkey", hlua::function1(move |pubkey: AnyLuaValue| -> Result<AnyLuaValue> {
        let pubkey = byte_array(pubkey)
            .map_err(|err| state.set_error(err))?;
        pgp_pubkey_lua(pubkey)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn pgp_pubkey_armored(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("pgp_pubkey_armored", hlua::function1(move |pubkey: String| -> Result<AnyLuaValue> {
        let mut r = BufReader::new(pubkey.as_bytes());
        let pubkey = sloppy_rfc4880::armor::read_armored(&mut r)
            .map_err(|err| state.set_error(err))?;

        pgp_pubkey_lua(pubkey)
            .map_err(|err| state.set_error(err))
    }))
}


#[cfg(test)]
mod tests {
    use engine::ctx::Script;

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

            if key['uids'][1] ~= "Hans Acker (example comment) <hans.acker@example.com>" then
                return "Unexpected uid: " .. key['uid']
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
