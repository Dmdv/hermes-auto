#!/usr/bin/env ruby
# frozen_string_literal: true

# This daemon will run periodically to update the proxy configuration

require 'daemons'

dirname = File.dirname(File.absolute_path(__FILE__))
filename = File.join(dirname, './hermes_many.rb proxy-update')
svcname = 'proxy-config'
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
    system(filename)
    sleep(60 * 60) # 1 hour
  end
end

