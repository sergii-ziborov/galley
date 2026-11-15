param(
    [string]$Ans,
    [string]$Png,
    [int]$Cols,
    [int]$Rows
)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
function Level([int]$i) {
    if ($i -le 0) { return 0 }
    $v = 55 + $i * 40
    if ($v -gt 255) { return 255 }
    return $v
}
function Color256([int]$n) {
    $basic = @(
        @(0,0,0), @(128,0,0), @(0,128,0), @(128,128,0),
        @(0,0,128), @(128,0,128), @(0,128,128), @(192,192,192),
        @(128,128,128), @(255,0,0), @(0,255,0), @(255,255,0),
        @(0,0,255), @(255,0,255), @(0,255,255), @(255,255,255)
    )
    if ($n -lt 0) { $n = 0 }
    if ($n -lt 16) { return $basic[$n] }
    if ($n -ge 232) {
        $v = 8 + ($n - 232) * 10
        if ($v -gt 255) { $v = 255 }
        return @($v, $v, $v)
    }
    $n -= 16
    $ri = [int][Math]::Floor($n / 36)
    $gi = [int][Math]::Floor(($n % 36) / 6)
    $bi = [int]($n % 6)
    return @((Level $ri), (Level $gi), (Level $bi))
}
function SextantMask([int]$cp) {
    # Bit 0 is the top left, then top right, middle left, middle right, bottom left, bottom right.
    if ($cp -eq 0x258C) { return 0x15 }
    if ($cp -eq 0x2590) { return 0x2A }
    if ($cp -eq 0x2588) { return 0x3F }
    if ($cp -eq 0x2580) { return 0x0F }
    if ($cp -eq 0x2584) { return 0x30 }
    if ($cp -ge 0x1FB00 -and $cp -le 0x1FB13) { return ($cp - 0x1FB00) + 1 }
    if ($cp -ge 0x1FB14 -and $cp -le 0x1FB27) { return ($cp - 0x1FB00) + 2 }
    if ($cp -ge 0x1FB28 -and $cp -le 0x1FB3B) { return ($cp - 0x1FB00) + 3 }
    return -1
}
$text = [System.IO.File]::ReadAllText($Ans, [System.Text.Encoding]::UTF8)
$cellW = 9
$cellH = 18
$bmp = New-Object System.Drawing.Bitmap ($Cols * $cellW), ($Rows * $cellH)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.Clear([System.Drawing.Color]::FromArgb(255, 8, 8, 8))
$font = New-Object System.Drawing.Font("Consolas", 14, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
$fg = [System.Drawing.Color]::FromArgb(255, 220, 200, 180)
$bg = [System.Drawing.Color]::FromArgb(255, 8, 8, 8)
$x = 0
$y = 0
$i = 0
$chars = $text.ToCharArray()
while ($i -lt $chars.Length) {
    $ch = $chars[$i]
    if ($ch -eq [char]27) {
        $i++
        if ($i -lt $chars.Length -and $chars[$i] -eq '[') {
            $i++
            $seq = ""
            while ($i -lt $chars.Length -and $chars[$i] -ne 'm' -and $chars[$i] -ne 'H' -and $chars[$i] -ne 'h' -and $chars[$i] -ne 'l' -and $chars[$i] -ne 'r') {
                $seq += $chars[$i]
                $i++
            }
            $end = if ($i -lt $chars.Length) { $chars[$i] } else { 'm' }
            $i++
            if ($end -eq 'H') {
                $parts = $seq.Split(';')
                $y = [Math]::Max(0, ([int]$parts[0]) - 1)
                $x = if ($parts.Length -gt 1) { [Math]::Max(0, ([int]$parts[1]) - 1) } else { 0 }
            } elseif ($end -eq 'm') {
                $parts = @()
                if ($seq.Length -gt 0) { $parts = $seq.Split(';') }
                $p = 0
                while ($p -lt $parts.Length) {
                    $code = 0
                    [void][int]::TryParse($parts[$p], [ref]$code)
                    if ($code -eq 0) {
                        $fg = [System.Drawing.Color]::FromArgb(255, 220, 200, 180)
                        $bg = [System.Drawing.Color]::FromArgb(255, 8, 8, 8)
                    } elseif ($code -eq 38 -and ($p + 2) -lt $parts.Length -and $parts[$p + 1] -eq "5") {
                        $n = [int]$parts[$p + 2]
                        $rgb = Color256 $n
                        $fg = [System.Drawing.Color]::FromArgb(255, $rgb[0], $rgb[1], $rgb[2])
                        $p += 2
                    } elseif ($code -eq 48 -and ($p + 2) -lt $parts.Length -and $parts[$p + 1] -eq "5") {
                        $n = [int]$parts[$p + 2]
                        $rgb = Color256 $n
                        $bg = [System.Drawing.Color]::FromArgb(255, $rgb[0], $rgb[1], $rgb[2])
                        $p += 2
                    } elseif ($code -eq 38 -and ($p + 4) -lt $parts.Length -and $parts[$p + 1] -eq "2") {
                        $fg = [System.Drawing.Color]::FromArgb(255, [int]$parts[$p + 2], [int]$parts[$p + 3], [int]$parts[$p + 4])
                        $p += 4
                    } elseif ($code -eq 48 -and ($p + 4) -lt $parts.Length -and $parts[$p + 1] -eq "2") {
                        $bg = [System.Drawing.Color]::FromArgb(255, [int]$parts[$p + 2], [int]$parts[$p + 3], [int]$parts[$p + 4])
                        $p += 4
                    } elseif ($code -eq 30) {
                        $fg = [System.Drawing.Color]::FromArgb(255, 12, 12, 12)
                    }
                    $p++
                }
            }
        } elseif ($i -lt $chars.Length -and $chars[$i] -eq ']') {
            while ($i -lt $chars.Length -and $chars[$i] -ne [char]7) { $i++ }
            $i++
        }
        continue
    }
    if ($ch -eq "`n" -or $ch -eq "`r") { $i++; continue }
    $cp = [int]$ch
    if ([char]::IsHighSurrogate($ch) -and ($i + 1) -lt $chars.Length -and [char]::IsLowSurrogate($chars[$i + 1])) {
        $cp = [char]::ConvertToUtf32($ch, $chars[$i + 1])
        $i++
    }
    if ($x -ge 0 -and $y -ge 0 -and $x -lt $Cols -and $y -lt $Rows) {
        $rect = New-Object System.Drawing.Rectangle ($x * $cellW), ($y * $cellH), $cellW, $cellH
        $brushBg = New-Object System.Drawing.SolidBrush $bg
        $g.FillRectangle($brushBg, $rect)
        $brushBg.Dispose()
        $mask = SextantMask $cp
        if ($mask -ge 0) {
            $fgBrush = New-Object System.Drawing.SolidBrush $fg
            $bgBrush = New-Object System.Drawing.SolidBrush $bg
            $g.FillRectangle($bgBrush, $rect)
            $tw = [int]($cellW / 2)
            $th = [int]($cellH / 3)
            for ($bit = 0; $bit -lt 6; $bit++) {
                if (($mask -band (1 -shl $bit)) -eq 0) { continue }
                $sx = ($x * $cellW) + (($bit % 2) * $tw)
                $sy = ($y * $cellH) + ([int]($bit / 2) * $th)
                $g.FillRectangle($fgBrush, $sx, $sy, $tw, $th)
            }
            $fgBrush.Dispose()
            $bgBrush.Dispose()
        } elseif ($ch -ne ' ') {
            $brush = New-Object System.Drawing.SolidBrush $fg
            $g.DrawString([string]$ch, $font, $brush, ($x * $cellW), ($y * $cellH) - 1)
            $brush.Dispose()
        }
    }
    $x++
    $i++
}
$bmp.Save($Png, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
$font.Dispose()
