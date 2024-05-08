while ($true)
{
    # Get a list of running processes
    $processes = Get-Process | Where-Object { $_.ProcessName -ne 'Idle' }

    # Display processes for selection
    Clear-Host
    $processes | Format-Table Id, ProcessName -AutoSize

    # Prompt for selection
    $processName = Read-Host "Enter the name of the process to monitor (or 'q' to quit)"

    # Validate selection and exit if requested
    if ($processName -eq 'q')
    {
        break
    }

    while (-not ($processes | Where-Object { $_.ProcessName -eq $processName }))
    {
        Write-Warning "Invalid selection. Please enter a valid process name."
        $processName = Read-Host "Enter the name of the process to monitor (or 'q' to quit)"
        if ($processName -eq 'q')
        {
            break
        }
    }

    if ($processName -eq 'q')
    {
        break
    } # Exit if 'q' was entered

    $prevTotalBytesSent = 0

    while ($true)
    {

        $process = Get-Process -Name $processName -ErrorAction SilentlyContinue
        if (-not $process)
        {
            Write-Warning "Process with name $processName not found."
            Start-Sleep -Seconds 5
            continue
        }

        # Current Time
        $currentTime = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        Write-Host "$currentTime | " -NoNewline

        # CPU Usage
        $cpuUsage = [Math]::Round(($process.CPU / ($process.TotalProcessorTime.Ticks / 10000)) * 100, 2)
        Write-Host "PS-CPU: $cpuUsage% | " -NoNewline

        # Total CPU Usage
        $totalCpuCounter = Get-Counter -Counter "\Processor(_Total)\% Processor Time" -ErrorAction SilentlyContinue
        if ($null -eq $totalCpuCounter)
        {
            Write-Host "CPU counter is not available for the system | "
        }
        else
        {
            $totalCpuUsage = [Math]::Round($totalCpuCounter.CounterSamples[0].CookedValue, 2)
            Write-Host "CPU: $totalCpuUsage% | " -NoNewline
        }

        # Memory Usage
        $memoryUsage = [Math]::Round($process.WorkingSet64 / 1MB, 2)
        Write-Host "PS-RAM-Usage: $memoryUsage MB | " -NoNewline

        # Disk Usage
        try
        {
            $diskCounters = Get-Counter -Counter "\LogicalDisk(_Total)\Disk Read Bytes/sec", "\LogicalDisk(_Total)\Disk Write Bytes/sec" -ErrorAction SilentlyContinue
            if ($null -eq $diskCounters)
            {
                Write-Host "Disk I/O counters are not available for the system | "
            }
            else
            {
                $readBytes = [Math]::Round($diskCounters.CounterSamples[0].CookedValue / 1KB, 2)
                $writeBytes = [Math]::Round($diskCounters.CounterSamples[1].CookedValue / 1KB, 2)
                Write-Host "Disk R: $readBytes KB/s | " -NoNewline
                Write-Host "Disk W: $writeBytes KB/s | " -NoNewline
            }
        }
        catch
        {
            Write-Warning "Failed to get disk I/O: $_ | "
        }

        # Network Usage
        $networkAdapters = Get-NetAdapter | Where-Object Status -eq "Up"
        $totalBytesSentStart = 0
        foreach ($adapter in $networkAdapters)
        {
            $statistics = Get-NetAdapterStatistics -Name $adapter.Name
            $totalBytesSentStart += $statistics.SentBytes
        }

        # Wait for a specific period of time (e.g., 1 second)
        Start-Sleep -Seconds 1

        # Calculate bytes sent again
        $totalBytesSentEnd = 0
        foreach ($adapter in $networkAdapters)
        {
            $statistics = Get-NetAdapterStatistics -Name $adapter.Name
            $totalBytesSentEnd += $statistics.SentBytes
        }

        # Calculate bytes sent during the time interval
        $bytesSent = $totalBytesSentEnd - $totalBytesSentStart

        # Convert bytes per second to Megabits per second (1 Byte = 8 bits, 1 Megabit = 10^6 bits)
        $networkSpeedMbps = ($bytesSent * 8) / 1e6

        Write-Host "NetworkOut: $networkSpeedMbps Mbps" -NoNewline

        Write-Host " "

        Start-Sleep -Seconds 1 # Update interval
    }
}