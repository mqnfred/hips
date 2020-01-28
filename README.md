# **H**ide **i**n **p**lain **s**ight

The goal of `hips` is to enable developers to store their production secrets
alongside their code.

While it is beneficial to treat secrets like code, it is not advisable to store
them alongside it in a plain text fashion. This is where `hips` comes in: it is
a small utility meant to manage a yaml file containing encrypted secrets.

## Manipulating secrets

Let's look at what a typical secret management session might look like:

```shell
$ cat store.yaml
cat: store.yaml: No such file or directory

$ hips -s store.yaml -p my_master_password set my_secret 'what-i-want-to-hide'
$ cat store.yaml
---
my_secret: gq8DEm5ot4eFQu/y+Y4UxwJ0RZ162EfaMJa2EuefXEk=

$ hips -s store.yaml -p my_master_password get my_secret
what-i-want-to-hide

$ hips -s store.yaml -p wrong_password get my_secret
TODO: error

$ hips -s store.yaml -p my_master_password env --interpreter '/bin/sh'
#!/bin/sh

export MY_SECRET = 'what-i-want-to-hide';
```

You can see that we expose three commands:

 - `set` add/overwrite a new secret
 - `get` read an existing secret
 - `env` expose secrets as environment variables in a shell script

While the first two are useful for management reasons, the last command is used
when programs need to load those secrets into memory, which is conveniently
done using environment variables.
