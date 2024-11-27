#!/usr/bin/env ruby
# frozen_string_literal: true

require 'json'
require 'net/http'

working_dir = File.dirname(File.absolute_path(__FILE__))
Dir.chdir(working_dir)

# timestamp = Time.now.strftime('%Y%m%d%H%M%S')
# log_name = "log-hermes-many-#{timestamp}.log".to_s

chains_reg = JSON.parse(File.read('phoenix-1-channels.json'))

channels = chains_reg['channels']

channels.each do |channel|
  target = channel['chain']['chain_id']
  client_id = channel['source']['client_id']
  next if client_id.nil? || client_id.empty?

  url = "https://phoenix-lcd.terra.dev/ibc/core/client/v1/client_status/#{client_id}"
  uri = URI(url)

  http = Net::HTTP.new(uri.host, uri.port)
  http.use_ssl = true

  request = Net::HTTP::Get.new(url)
  response = http.request(request)
  json_response = JSON.parse(response.body)

  puts "[#{json_response['status']}] Terra2 -> #{target} Client ID: #{client_id}"
end
