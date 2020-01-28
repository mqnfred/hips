# Hide in plain sight

The goal of `hips` is to enable developers to store their production secrets
alongside their code.

While it is beneficial to treat secrets like code, it is not advisable to store
them alongside it in a plain text fashion. `hips` is a small utility meant to
manage a yaml file containing **encrypted** secrets.

## Why? me+code=secrets

Most of the big shops out there need to roll out their secret managers, in part
because they are higher-profile targets and because secrets cannot be tied to
individuals anymore. You can probably find an AWS that does that.

For small shops who do not want to marry into any cloud provider however,
tracking secrets is a weird exercise. We suggest tracking them alongside the
code, which is possible thanks to the small scale.

By treating secrets as code, we reduce the source of truths in our distributed
systems by one. It also contributes to helping us design our infrastructure as
code in our repo by making access to secrets easier. We'll call it minimalistic
devops!

This might only be possible at a certain scale however, as the master
password concept is probably not sustainable past a certain amount of people.

## Example: manipulating secrets

Let's look at what a typical secret management session might look like:

```shell
$ echo my-master-pw | hips -d secrets.yaml set my_secret 'what-i-want-to-hide'
$ cat secrets.yaml
---
my_secret: GSA2NIQ+ox2PpyzKha9g+qVWj+MwrwBAOClA8sqOW7qLdIaU0tKCli78yfjj/0k=

$ echo my-master-pw | hips -d secrets.yaml get my_secret
what-i-want-to-hide

$ echo bad-pw | target/release/hips -d secrets.yaml get my_secret
error: retrieving secret: decrypting secret: processing ciphertext: OpenSSL error

$ echo my-master-pw | hips -d secrets.yaml env --shell=/bin/bash
#!/bin/bash
export MY_SECRET='what-i-want-to-hide';
```

We expose four commands currently:

 - `set` add/overwrite a new secret
 - `get` read an existing secret
 - `rot` re-encrypts the whole database using a new password
 - `env` expose secrets as environment variables in a shell script

While the first two are useful for management reasons, the last command is used
when programs need to load those secrets into memory, which is conveniently
done using environment variables.

## Safety

Everyone will not be comfortable with this and that's ok. Storing your secrets
encrypted next to your code means you need to trust the entity protecting your
code in the first place.

I personally think that this is fine, and it is likely that anybody able to
temper with my code without my noticing would thereby be able to get me to
execute arbitrary things and ultimately get access to my production.

Ultimately, consider the following two questions:

 - Your profile (are you a high-profile target? low-profile?)
 - Your threat-level (who do you accept to trust?)

It is important to answer those before making any decision regarding security,
and if possible, consult with a security engineer. We rely on openssl's
`aes256` to encrypt/decrypt and `pbkdf2` to derive a proper key from a
password.

If you know anything about brute-forcing those ciphers that I don't, please
contact me. Also reach out if you have hindsight into malpractices or
untrustworthiness from the usual source code providers. I am open to being
convinced that this is unsafe.
