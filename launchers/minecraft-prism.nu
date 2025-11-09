# Provides the [Minecraft] Prism Launcher integration.

# Extract the unit properties from [Minecraft] Prism Launcher.
#
# Returns `null` if the launcher environment is not [Minecraft] Prism Launcher.
export def main []: nothing -> oneof<list<string>, nothing> {
	let INSTANCE_ID: string = (
		$env.INST_ID?
		| if $in != null { $in } else { return null }
	)
	let INSTANCE_NAME: string = (
		$env.INST_NAME?
		| if $in != null { $in } else { return null }
	)

	[
		'-a' $"minecraft_prism_($INSTANCE_ID)",
		'-d' $"Minecraft \(($INSTANCE_NAME)\)",
	]
}
