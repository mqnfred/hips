# Hide in plain sight

`hips` is a small, self-contained utility that enables users to store their
secrets encrypted alongside their code. What are you interested in knowing?

 1. [Why do this?](#philosophy)
 2. [Let's try it out](#install)
 3. [Tutorial](#tutorial)
 4. [Is this even safe](#safety)

## Philosophy

`me+code=secrets`

For all the small shops out there, the low profile targets with a limited
amount of individual developers involved, we suggest tracking your secrets
alongside your code, in a file or folder database.

This will reduce sources of truth in your distributed system by 1 and help with
"infrastructure as code" by making access to the secrets a local affair. You
will not have to get married to some cloud provider's secret manager, you will
not have to deploy an open-source solution yourself.

For teams with many developers, the master-password strategy does not scale
well. For higher scale infrastructure and higher-profile shops, managed secrets
will likely make more sense.

## Install

You will need [cargo][1] to install `hips`. Once you have it, do the following:

```sh
export PATH=$PATH:$HOME/.cargo/bin
cargo install hips
hips --help
```

## Tutorial

In this tutorial you will learn about all the different commands and database
formats that `hips` supports.

### Database and password configuration

We use environment variables to pass the database and password to `hips`:

```
$ export HIPS_DATABASE=secrets.yaml
$ export HIPS_PASSWORD=pw
```

### Store, Load and Remove

`store` takes a name and a secret and stores them in the database.

```
$ hips store aws_access_key_id BUIO1IXUAK3OQ9ACAHSX
$ hips store aws_secret_access_key UwioixhaklufhhWbaXoSLwbxb2dj7/AJs92bdsXh
$ cat secrets.yaml
---
- name: aws_access_key_id
  secret: c58C04qkTDQhg86piVlmg7EXcz66i3C3GSdHjmZW5v2Pa6Froo69gbuDNSICXh4w
  salt: OH3/mX3e/3ODwSLBYzFiK92PztSgLLmIf5S8mqenwXo=
- name: aws_secret_access_key
  secret: asrhBl8bMTPGj8Cua6LzyseRBLmhmNrivaCjW53NcNRUyKSYOkoLdq9PHSPHKdgosO6/acOn3hv+vnkciwLj0tio0ac=
  salt: 4GP2GtoRhaf6NKtanBm9aLjUefuNH+otFDFfHF1Utns=%
$ hips remove aws_access_key_id
```

`load` takes a name and prints out the matching secret.

```
$ hips load aws_access_key_id
BUIO1IXUAK3OQ9ACAHSX
$ hips load aws_secret_access_key
UwioixhaklufhhWbaXoSLwbxb2dj7/AJs92bdsXh
```

`remove` (`rm`) takes a name and removes that secret from the database.

```
$ hips store remove_me_soon unimportant-secret
$ cat secrets.yaml | grep name:
- name: aws_access_key_id
- name: aws_secret_access_key
- name: remove_me_soon
$ hips remove remove_me_soon
$ cat secrets.yaml | grep name:
- name: aws_access_key_id
- name: aws_secret_access_key
```

### Rotate

You can `rotate` (`rot`) the secrets database in one command, re-encrypting
everything using a different password.

```
$ cat secrets.yaml | grep secret:
  secret: c58C04qkTDQhg86piVlmg7EXcz66i3C3GSdHjmZW5v2Pa6Froo69gbuDNSICXh4w
  secret: asrhBl8bMTPGj8Cua6LzyseRBLmhmNrivaCjW53NcNRUyKSYOkoLdq9PHSPHKdgosO6/acOn3hv+vnkciwLj0tio0ac=
$ hips rotate new-pw
$ cat secrets.yaml | grep secret:
  secret: Sb8BznQqjYr+q+lis2uVKPZ/j+qmNIMuXbjr/MElIAYkupyUCGHPbY+N/NTpTxKr
  secret: FlWQvkzidFa8mStgoEbQXt4MobHPQFT7NFnImKIjgfNZ7xFhPGjj3kD0z3x4YNzLTjBMJykk57JooYCojhOH/GlqeEk=
$ export HIPS_PASSWORD=new-pw
$ hips load aws_access_key_id
BUIO1IXUAK3OQ9ACAHSX
```

You can see here that the encrypted secrets and salt are different from the
previous `secrets.yaml` database. We can now read all the secrets using the new
password.

### Template

Many times when exporting secrets to production, they need to be displayed in a
specific manner, as part of a configuration file and such. We use the [tiny
template][2] library for this purpose. See their neat [syntax page][3] for more
information. We will cover our templating capabilities in the following two
examples.

#### AWS credentials file

Let's generate the `.aws/credentials`, where amazon secrets are conventionally
stored. We'll use the "map" feature of the templating framework, which allows
us to print out specific secrets by name.

```
$ hips template '[default]\naws_access_key_id={map.aws_access_key_id}\naws_secret_access_key={map.aws_secret_access_key}'
[default]
aws_access_key_id=BUIO1IXUAK3OQ9ACAHSX
aws_secret_access_key=UwioixhaklufhhWbaXoSLwbxb2dj7/AJs92bdsXh
```

#### Shell script loading all secrets

This time, since our template is a bit more complex, we'll store it in a file:

```
$ cat shell-template
#!/bin/sh
{{ for secret in list -}}
{{- if not @first }}\n{{ endif -}}
export {secret.name|capitalize}={secret.secret};
{{- endfor -}}
```

You can find more information about this syntax [here][3]. This will yield the
following shell script:

```
$ hips template shell-template
#!/bin/sh
export AWS_ACCESS_KEY_ID=BUIO1IXUAK3OQ9ACAHSX;
export AWS_SECRET_ACCESS_KEY=UwioixhaklufhhWbaXoSLwbxb2dj7/AJs92bdsXh;
```

### Database formats

Up until now, we have been using a yaml file as database. We support multiple
formats however:

 - As a directory hierarchy (no extension)
 - As a single yaml file (`.yaml` extension)

If we were to repeat the experiment above with a directory hierarchy under
`secrets/`, our database would look like this:

```
$ tree secrets/
secrets
├── aws_access_key_id/
│   ├── salt
│   └── secret
└── aws_secret_access_key/
    ├── salt
    └── secret
$ cat secrets/aws_access_key_id/secret
Sb8BznQqjYr+q+lis2uVKPZ/j+qmNIMuXbjr/MElIAYkupyUCGHPbY+N/NTpTxKr
```

## Safety

This is as safe as SSL and HTTPS. It uses the same encryption technologies and
the same libraries. If you feel safe with SSL and HTTPS, you should, in theory
at least, feel safe about storing encrypted secrets alongside your code.

You might want to consider some burden of trust (in your threat-model) on the
entity hosting your code, if you want to be conservative. This will not matter
materially however when compared with the burden that lies with the encryption.

Ultimately, consider the following two characteristics:

 - Your profile (are you a high-profile target? low-profile?)
 - Your threat-level (who do you accept to trust?)

It is important to answer those questions before making any decision regarding
security. If possible, consult with some engsec. On the technical side, we rely
on openssl's `aes256` to encrypt/decrypt and `pbkdf2` to derive a proper key
from a password.

[1]: https://crates.io
[2]: https://crates.io/crates/tinytemplate
[3]: https://docs.rs/tinytemplate/1.0.4/tinytemplate/syntax/index.html
