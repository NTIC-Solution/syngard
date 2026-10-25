# Fixtures

Inert corpus. Nothing here is executed by the scanner or the tests: scripts
print fixed strings, hosts use reserved domains (`example.invalid`), and any
credential-like value is fake. Tests copy fixtures to a temporary directory
before mutating them.
