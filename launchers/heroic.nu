# Provides the Heroic Launcher integration.

# Extract the unit properties from Heroic Launcher (Legendary).
#
# Returns `null` if the launcher environment is not Heroic Launcher (Legendary).
# Throws an error if the game's metadata cannot be found.
def "main legendary" []: nothing -> oneof<list<string>, nothing, error> {
	let APP_ID: string = (
		$env.HEROIC_APP_NAME?
		| if $in != null { $in } else { return null }
	)
	let CONFIG_PATH: path = (
		$env.LEGENDARY_CONFIG_PATH?
		| if $in != null { $in } else { return null }
	)

	let metadata: path = $CONFIG_PATH | path join 'metadata' $"($APP_ID).json"
	if ($metadata | path exists) {
		let metadata: record = (
			$metadata
			| open --raw
			| from json
		)

		[
			'-a' $"heroic_legendary_($APP_ID)",
			'-d' $metadata.app_title,
		]
	} else {
		error make --unspanned {
			msg: $"could not find the metadata for the Legendary app ($APP_ID)",
		}
	}
}

# Extract the unit properties from Heroic Launcher (GOGDL).
#
# Returns `null` if the launcher environment is not Heroic Launcher (GOGDL).
# Throws an error if the game's metadata cannot be found.
def "main gogdl" []: nothing -> oneof<list<string>, nothing, error> {
	let APP_ID: string = (
		$env.HEROIC_APP_NAME?
		| if $in != null { $in } else { return null }
	)
	let CONFIG_PATH: path = (
		$env.XDG_CONFIG_HOME?
		| default ($nu.home-path | path join '.config')
		| path join 'heroic'
	)

	let library: path = $CONFIG_PATH | path join 'store_cache' 'gog_library.json'
	if ($library | path exists) {
		let library: table = (
			$library
			| open --raw
			| from json
			| get games
		)

		let game = $library | where app_name == $APP_ID | first
		if $game != null {
			[
				'-a' $"heroic_gogdl_($APP_ID)",
				'-d' $game.title,
			]
		} else {
			error make --unspanned {
				msg: $"could not find the metadata for the GOGDL app ($APP_ID)",
			}
		}
	} else {
		error make --unspanned {
			msg: $"could not find the library for GOG",
		}
	}
}

# Extract the unit properties from Heroic Launcher.
#
# Returns `null` if the launcher environment is not Heroic Launcher.
# Throws an error if the game's metadata cannot be found.
export def main []: nothing -> oneof<list<string>, nothing, error> {
	match $env.HEROIC_APP_RUNNER? {
		'legendary' => (main legendary),
		'gog' => (main gogdl),
		_ => null,
	}
}
