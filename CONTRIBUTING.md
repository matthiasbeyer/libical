# Contributing to libical

So you want to contribute to libical! Thank you, that's awesome!

All contributors agree to the
[developer certificate of origin](#developer-certificate-of-origin)
by contributing to libical.

Feel free to contact [us via our mailinglist](http://imag-pim.org/mailinglist/)
and/or submit patches via mail (use `git format-patch` and
`git send-email`, always add a cover letter to describe your submission).
You don't have to send patches via mail, though. As long as I can `git pull`
your changes (without having to login or register at the remote) or `git am`
your patchset, I'm fine.
I'd encourage you, though, to [use git-send-email](https://git-send-email.io)
or at least [git-request-pull](https://git-scm.org/).

Make sure to test-compile your patchset and run tests if there are any for the
code you changed.

I will run your patchset through travis CI.


## Prerequisites

You'll need libical installed on your system to test-compile this library.
If you have a `nix` installation, you can issue
`nix-shell --run "cargo check --tests"`
to check the library.


## Commit guidelines

Make sure your patchset does not contain "Fixup" commits when publishing it, but feel
free to send  "Fixup" commits in the review process. If squashing fails I will
come back to you.

Also ensure that each commit has
[a "Signed-off-by: " line](https://stackoverflow.com/questions/1962094/what-is-the-sign-off-feature-in-git-for).
By adding that line, you agree to our
[developer certificate of origin](#developer-certificate-of-origin).
If you do not add the "Signed-off-by: " line, I reserve the right to kindly
reject your patch.


## Code of Conduct

We use the same
[code of conduct as the rust community does](https://www.rust-lang.org/conduct.html).

Basically: Be kind, encourage others to ask questions - you are encouraged to
ask questions as well!


## Developer Certificate of Origin

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.
660 York Street, Suite 102,
San Francisco, CA 94110 USA

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```


