# Hide in plain sight

[![crates.io](https://img.shields.io/crates/v/hips)](https://crates.io/crates/hips)

`hips` is a small, self-contained utility that enables users to store their
secrets encrypted alongside their code. It can be used as a binary or a
[library][7] depending on your needs. What are you interested in knowing?

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
will not have to depend on any kind of remote infrastructure for your secrets.

This solution will not work well for you if you:

 - Have many developers (master-password strategy does not scale well)
 - High scale/complexity infra (a need for secrets as a service)
 - High profile shops (compliance reasons, need insurances)
 
In this database, you could store your AWS credentials or the ssh key you use
to connect to your production. You could store secrets needed by your serving
layer to authenticate with your database and push those using a tool like
ansible or ssh.

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

### Store, Load, List, Remove, Rename

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
```

`load` takes a name and prints out the matching secret.

```
$ hips load aws_access_key_id
BUIO1IXUAK3OQ9ACAHSX
$ hips load aws_secret_access_key
UwioixhaklufhhWbaXoSLwbxb2dj7/AJs92bdsXh
```

`list` prints the names of the secrets stored in the db sorted alphabetically.

```
$ hips ls
aws_access_key_id
aws_secret_access_key
```

`remove` (`rm`) takes a name and removes that secret from the database.

```
$ hips store remove_me_soon unimportant-secret
$ hips ls
aws_access_key_id
aws_secret_access_key
remove_me_soon
$ hips remove remove_me_soon
$ hips ls
aws_access_key_id
aws_secret_access_key
```

`rename` renames a secret based on a (orig, dest) name pair

```
$ hips store move_me_please unimportant-secret
$ hips ls
aws_access_key_id
aws_secret_access_key
move_me_please
$ hips rename move_me_please thats_better
$ hips ls
aws_access_key_id
aws_secret_access_key
thats_better
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

This project is using [ring][4]'s `pbkdf2` function to derive a proper key from
a password and its `aes256` implementation to encrypt/decrypt the secrets. In
theory at least, those ciphers should not be brute-forceable. You can find an
audit of the ring library by Cure53 [here][6].

With this being said, it is still important to protect the encrypted version of
our secrets from being public. If you store your secrets alongside your code,
that responsibility then befalls your code provider (github for example.)

Ultimately, consider the following three characteristics:

 - Your profile (low-profile target, high-profile?)
 - Your threat-model (who do you accept to trust?)
 - Your compliance requirements (do you have [PII][5] data?)

You need to consider all those questions (and more) before deciding on a
solution for your secrets management. It is advised that you consult with a
security engineer as well.

[1]: https://crates.io
[2]: https://crates.io/crates/tinytemplate
[3]: https://docs.rs/tinytemplate/1.0.4/tinytemplate/syntax/index.html
[4]: https://github.com/briansmith/ring
[5]: https://en.wikipedia.org/wiki/Personal_data
[6]: https://github.com/ctz/rustls/blob/master/audit/TLS-01-report.pdf
[7]: https://docs.rs/hips
