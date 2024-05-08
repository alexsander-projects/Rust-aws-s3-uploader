Get-ChildItem | ForEach-Object {
    $newName = $_.BaseName + "_copy" + $_.Extension
    if (Test-Path -Path $newName) {
        # If a file with the new name exists, add a number to differentiate 
        $i = 1
        while (Test-Path "$newName($i)") {
            $i++
        }
        $newName = $_.BaseName + "_copy($i)" + $_.Extension
    }

    # Duplicate the item:
    if ($_.PSIsContainer) {
        # It's a folder
        Copy-Item $_.FullName $newName -Recurse -Force
    } else {
        # It's a file
        Copy-Item $_.FullName $newName -Force
    }
}