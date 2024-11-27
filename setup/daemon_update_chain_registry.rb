#!/usr/bin/env ruby
# frozen_string_literal: true

require 'daemons'

dirname = File.dirname(File.absolute_path(__FILE__))
filename = File.join(dirname, 'update_chain_registry.rb')
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
    sleep(60 * 60 * 24)
  end
end

