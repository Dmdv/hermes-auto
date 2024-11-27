#!/usr/bin/env ruby
# frozen_string_literal: true

# This daemon will run periodically create_channels.rb which creates new channels if they appear in chain_registry

require 'daemons'

dirname = File.dirname(File.absolute_path(__FILE__))
filename = File.join(dirname, 'create_channels.rb')
svcname = File.basename(filename, '.rb')
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
    puts "Running #{filename}"
    system('ruby', filename)
    sleep(60 * 60 * 3) # 3 hours
  end
end

