$path = Join-Path $pwd "test.txt"
$size = 1GB
$content = New-Object byte[] $size
(New-Object System.Random).NextBytes($content)

# Set-Content is very slow, use .NET method directly
[System.IO.File]::WriteAllBytes($path, $content)