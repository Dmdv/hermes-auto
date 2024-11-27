#!/usr/bin/env ruby
# frozen_string_literal: true

# This daemon will run periodically to update the proxy configuration

require 'daemons'

dirname = File.dirname(File.absolute_path(__FILE__))
start_proc = File.join(dirname, './hermes-many.rb start -a -p')
stop_proc = File.join(dirname, './hermes-many.rb stop')
svcname = 'hermes-supervisor'
Dir.chdir(dirname)

options = {
  app_name: svcname,
  dir_mode: :normal,
  monitor: true,
  monitor_interval: 5,
  log_output: true,
  pid_delimiter: '.n'
}

Daemons.run_proc(svcname, options) do
  loop do
    puts 'Stopping all hermes instances...'
    system(stop_proc)
    sleep 10
    puts 'Starting all hermes instances...'
    system(start_proc)
    puts 'Waiting next round...'
    sleep(60 * 60 * 3) # 1 hour
  end
end

