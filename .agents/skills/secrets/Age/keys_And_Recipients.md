# Age: Keys and Recipients

Sops, raw age and fnox all encrypt with native age keys, so one private key per machine and one list of recipients serve all three. This file owns where the key lives, which devices are recipients, the recipient list in the format each tool wants, and how a device joins or leaves. The tool-specific re-encryption steps stay in each tool's own doc.

## Key location

The canonical private key is `~/.config/age/age.txt` with `chmod 600`. Fnox and sops read it through symlinks, so no second key file exists. On a new machine, create the key once and the symlinks:

```bash
mkdir -p ~/.config/age ~/.config/fnox ~/.config/sops/age
age-keygen -o ~/.config/age/age.txt            # prints the age1... public key
chmod 600 ~/.config/age/age.txt
ln -sf ~/.config/age/age.txt ~/.config/fnox/age.txt
ln -sf ~/.config/age/age.txt ~/.config/sops/age/keys.txt
```

To print the public key of an existing machine later: `age-keygen -y ~/.config/age/age.txt`. The private key never leaves the machine and is never committed; only the `age1...` public key is shared.

## Known recipients (our devices)

| Device | Public key |
|---|---|
| jav (local, `javPc`) | `age1uy9ps3p4460de20v8fgvt6gyg5ml0wscesd32m4693567aaumgnsspt8x5` |
| g12 (server, `mail.zetunweb.com`) | `age1areaucgxqcwvnsnr3mpz46g27pc9gf8qqcvaaa3k5fn3fluvde3s27cql9` |

Any new encrypted file should include both so it opens on either machine.

**Sops** (`age:` value in `.sops.yaml`, comma-separated):

```yaml
age: >-
  age1uy9ps3p4460de20v8fgvt6gyg5ml0wscesd32m4693567aaumgnsspt8x5,age1areaucgxqcwvnsnr3mpz46g27pc9gf8qqcvaaa3k5fn3fluvde3s27cql9
```

**Fnox** (`[providers]` block in `fnox.toml`):

```toml
[providers]
age = { type = "age", recipients = [
    "age1uy9ps3p4460de20v8fgvt6gyg5ml0wscesd32m4693567aaumgnsspt8x5",  # jav (local)
    "age1areaucgxqcwvnsnr3mpz46g27pc9gf8qqcvaaa3k5fn3fluvde3s27cql9",  # g12 (server)
] }
```

**Raw age** (one `-r` per key):

```bash
age -r age1uy9ps3p4460de20v8fgvt6gyg5ml0wscesd32m4693567aaumgnsspt8x5 \
    -r age1areaucgxqcwvnsnr3mpz46g27pc9gf8qqcvaaa3k5fn3fluvde3s27cql9 \
    -o secrets.toml.age secrets.toml
```

## Adding a device

1. Create the key and symlinks on the new machine (above) and note its public key.
2. Add the public key to the recipients of every encrypted set, then re-encrypt:
   - sops: append it to the `age:` list in `.sops.yaml`, then `sops updatekeys` on the affected files ([Sops workflow](../Sops/workflow.md)).
   - fnox: add it to `recipients` in `fnox.toml`, then `fnox reencrypt -p age` ([Fnox installation](../Fnox/installation.md)).
   - raw age: add it to the recipient list, then decrypt and encrypt again ([Raw age workflow](workflow.md)).
3. Add the device to the table above and commit.

## Removing a device

Remove its key from the same lists and re-encrypt. This does not revoke access to ciphertext already in git history: anyone holding the old private key can still decrypt earlier versions. Rotate the secrets themselves when a device is lost or untrusted.
