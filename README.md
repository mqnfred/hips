# Hide in plain sight

The goal of `hips` is to enable developers to store their production secrets
alongside their code.

While it is beneficial to treat secrets like code, it is not advisable to store
them alongside it in a plain text fashion. This is where `hips` comes in: it is
a small utility meant to manage a yaml file containing encrypted secrets.

## Why?

As a software engineer, we have all seen mis-managed secrets. I can easily
imagine what it was like to manage code without versioning systems, because I
see it done with secrets.

Most of the big shops out there obviously roll out their secret managers, and
you can probably find that on AWS and the like. For the small shops however,
tracking secrets alongside the code is a gain in simplicity.

By treating secrets as code, we reduce the source of truths in our distributed
systems by one. It also contributes to helping us design our infrastructure as
code in our repo by making access to secrets easier. We'll call it minimalistic
devops!

## How it works

Somewhere in your repo, store your `secrets.yaml` database file. You can then
add and remove secrets using your master password with the `hips`' commands
`set` and `get`. When appropriate, you use `hips` commands to insert selected
secrets in the right places.

There also exist helpers which dump selected secrets into different formats.
For example, `env` dumps the contents of `secrets.yaml` in the form of a shell
script exporting environment variables.

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
error: retrieving secret: decrypting secret: decrypting ciphertext: OpenSSL error
```

You can see that we expose three commands:

 - `set` add/overwrite a new secret
 - `get` read an existing secret
 - `env` expose secrets as environment variables in a shell script

While the first two are useful for management reasons, the last command is used
when programs need to load those secrets into memory, which is conveniently
done using environment variables.

## Design decisions

We have chosen yaml because it is easily tracked by versioning systems,
minimalist, editable by hand and fairly ubiquitous. We rely on openssl's
`aes256` to encrypt/decrypt and `pbkdf2` to derive a proper key from a
password.
