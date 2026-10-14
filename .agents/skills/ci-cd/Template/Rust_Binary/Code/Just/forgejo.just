# Publishing triggers Woodpecker but never commits files automatically.

expected_origin := '__SET_FORGEJO_REMOTE_URL__'

[private]
publish-preflight:
    @test "$(git branch --show-current)" = 'main' || { echo 'Refusing to publish: the current branch is not main.' >&2; exit 1; }
    @test -z "$(git status --porcelain)" || { echo 'Refusing to publish: commit the intended working-tree changes first. This recipe never commits automatically.' >&2; git status --short >&2; exit 1; }
    @test '{{ expected_origin }}' != '__SET_FORGEJO_REMOTE_URL__' || { echo 'Refusing to publish: set expected_origin in Code/Just/forgejo.just.' >&2; exit 1; }
    @test "$(git remote get-url origin 2>/dev/null)" = '{{ expected_origin }}' || { echo 'Refusing to publish: origin does not match the reviewed Forgejo repository.' >&2; exit 1; }

[doc('Validate committed main, run the local CI gate, and push to trigger Woodpecker.')]
[group('forgejo')]
publish: publish-preflight
    just check
    git push -u origin main
