# Provides the Steam integration.

# Deserialize an ACF string.
def "from acf string" []: string -> record<value: string, rest: string> {
	mut rest = $in | str trim

	let value = (
		$rest
		| parse --regex '^(?<value>"[^"]*[^"\\]?")'
		| get 0.value
		| from json
	)
	$rest = $rest | str substring (($value | str length) + 2)..

	{
		value: $value,
		rest: $rest,
	}
}

# Deserialize an ACF object.
def "from acf object" [--root]: string -> record<value: record, rest: string> {
	mut rest = $in
	mut object = {}

	loop {
		$rest = $rest | str trim
		if ($root and ($rest | is-empty)) {
			break
		}
		if ($rest | str starts-with '}') {
			$rest = $rest | str substring 1..
			break
		}

		let res = $rest | from acf string
		$rest = $res.rest
		let key = $res.value

		$rest = $rest | str trim
		let value = if ($rest | str starts-with '{') {
			$rest = $rest | str substring 1..
			let res = $rest | from acf object
			$rest = $res.rest
			$res.value
		} else {
			let res = $rest | from acf string
			$rest = $res.rest
			$res.value
		}
		$object = $object | upsert $key $value
	}

	{
		value: $object,
		rest: $rest,
	}
}

# Convert from ACF to structured data.
def "from acf" []: string -> record {
	from acf object --root | get value
}

# Extract the unit properties from Steam.
#
# Returns `null` if the launcher environment is not Steam.
# Throws an error if the game's manifest cannot be found.
export def main []: nothing -> oneof<list<string>, nothing, error> {
	let APP_ID: string = (
		$env.STEAM_COMPAT_APP_ID?
		| if $in != null { $in } else { return null }
	)
	let LIBRARY_PATHS: list<path> = (
		$env.STEAM_COMPAT_LIBRARY_PATHS?
		| if $in != null { $in } else { return null }
		| split row (char esep)
	)

	for library in $LIBRARY_PATHS {
		let manifest: path = $library | path join $"appmanifest_($APP_ID).acf"
		if ($manifest | path exists) {
			let manifest: record = (
				$manifest
				| open --raw
				| from acf
				| get AppState
			)

			return [
				'-a' $"steam_($APP_ID)",
				'-d' $manifest.name,
			]
		}
	}
	error make --unspanned {
		msg: $"could not find the manifest for the Steam app ($APP_ID)",
	}
}
