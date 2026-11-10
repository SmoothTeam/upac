error = Error

err-read = Read failed
err-write = Write failed
err-malformed = Malformed PKI data
err-invalid-signature = Invalid signature
err-generation = Certificate generation failed

signature-valid = Signature valid

clap-usage = Usage
clap-options = Options
clap-arguments = Arguments
clap-commands = Commands
clap-help = Print help
clap-version = Print version
clap-help-subcommand = Print this message or the help of the given subcommand(s)
clap-error = error
clap-error-try-help = For more information, try '{ $help }'.
clap-error-missing-argument = the following required arguments were not provided: { $arguments }
clap-error-invalid-value = invalid value '{ $value }' for '{ $argument }'
clap-error-unknown-argument = unexpected argument '{ $argument }' found
clap-error-unknown-subcommand = unrecognized subcommand '{ $subcommand }'
clap-error-missing-subcommand = '{ $command }' requires a subcommand but one was not provided
clap-error-conflict = the argument '{ $argument }' cannot be used with '{ $prior }'

about = Sign and verify upac hook files with Ed25519 certificates
about-generate-root = Create a self-signed root certificate and its private key
arg-generate-root-common-name = Common name (CN) of the root certificate
arg-generate-root-key-out = Where to write the root private key
arg-generate-root-cert-out = Where to write the root certificate
about-generate-cert = Issue a signing certificate from a root certificate
arg-generate-cert-common-name = Common name (CN) of the new certificate
arg-generate-cert-root-key = Root private key used to sign the certificate
arg-generate-cert-root-cert = Root certificate that issues the certificate
arg-generate-cert-key-out = Where to write the new private key
arg-generate-cert-cert-out = Where to write the new certificate
about-sign-hook = Sign a hook file
arg-sign-hook-hook = Hook file to sign
arg-sign-hook-key = Private key of the signing certificate
arg-sign-hook-cert = Signing certificate
arg-sign-hook-signature = Where to write the signature
about-verify-hook = Verify a hook file's signature
arg-verify-hook-hook = Hook file to verify
arg-verify-hook-signature = Signature file of the hook
arg-verify-hook-root-cert = Root certificate the signing certificate must chain to
