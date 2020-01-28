# Hide in plain sight

The goal of `hips` is to enable developers to store their production secrets
alongside their code.

While it is beneficial to treat secrets like code, it is not advisable to store
them alongside it in a plain text fashion. This is where `hips` comes in: it is
a small utility meant to manage a yaml file containing encrypted secrets.

## Manipulating secrets

Let's look at what a typical secret management session might look like:

```shell
$ echo my-master-pw | hips -d secrets.yaml set my_secret 'what-i-want-to-hide'
$ cat secrets.yaml
---
my_secret: GSA2NIQ+ox2PpyzKha9g+qVWj+MwrwBAOClA8sqOW7qLdIaU0tKCli78yfjj/0k=
$ echo my-master-pw | hips -d secrets.yaml get my_secret
what-i-want-to-hide
```

You can see that we expose three commands:

 - `set` add/overwrite a new secret
 - `get` read an existing secret
 - `env` expose secrets as environment variables in a shell script

While the first two are useful for management reasons, the last command is used
when programs need to load those secrets into memory, which is conveniently
done using environment variables.
