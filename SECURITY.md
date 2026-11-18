# Reporting a security issue

This policy covers rust-edu.org, its static site generator and preview server, and the workflow that
builds and deploys the site. We maintain the site from `main`. If you found an issue on an older
checkout, include its commit so we can determine whether the current site or generator is affected.

## How to reach us

Open the repository's [Security tab](https://github.com/rust-edu/rust-edu.github.io/security).
If **Report a vulnerability** is available, use the form to contact the maintainers privately.
Otherwise, email [contact@rust-edu.org](mailto:contact@rust-edu.org) with the subject
“Security contact request” and ask for a private channel before sending technical details.
Keep vulnerability details out of public issues, pull requests, and chat.

In the private report, include the page URL or commit, the affected component, what you observed,
and who could be affected. Remove credentials and personal information from examples and logs.

## How maintainers handle reports

1. Assign one maintainer to acknowledge the report, coordinate the fix, and give the reporter a
   date for the next update. Aim to acknowledge new reports within three business days.
2. Determine whether the issue affects live visitors, contributors building or previewing the site,
   or the deployment workflow. If a credential may be exposed, revoke or rotate it promptly. If a
   deployment may be compromised, contain the incident and verify the source of the published site.
3. Coordinate sensitive fixes in a draft repository security advisory and a private fork. Add a
   regression test for the affected behavior. Run `make check` and `make ci` before merging the fix.
4. Verify the fixed commit or, for a live-site issue, the deployed site. Notify the reporter and
   coordinate disclosure. When users need to act, publish the affected commits or versions, the
   fix, and any available workaround. Ask the reporter before crediting them by name.
