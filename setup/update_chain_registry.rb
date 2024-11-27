#!/usr/bin/env ruby
# frozen_string_literal: true

# This script will create a dictionary chain -> chain_id from chain registry

require 'octokit'
require 'json'
require 'logger'

# frozen_string_literal: true
file_dirname = File.dirname(File.absolute_path(__FILE__))
Dir.chdir(file_dirname)

timestamp = Time.now.strftime('%Y%m%d%H%M%S')
log_name = "log-update_chain_registry-#{timestamp}.log".to_s

logger = Logger.new(log_name)
logger.formatter = proc do |severity, datetime, _progname, msg|
  "[#{datetime.strftime('%Y-%m-%d %H:%M:%S')}] #{severity}: #{msg}\n"
end

token = ''
repo = 'cosmos/chain-registry'
json_file = 'chain_chainid.json'

logger.info('Starting update_chain_registry daemon...')

client = Octokit::Client.new(access_token: token)
contents = client.contents(repo, path: '/')
system('rm', '-rf', json_file)

reg = {}

# Loop through the contents and print the name of each file
contents.each do |item|
  if item.type == 'file'
    logger.info item.name
  elsif item.type == 'dir'
    subfolder_contents = client.contents(repo, path: item.path)
    subfolder_contents.each do |subitem|
      next unless subitem.type == 'file'
      next unless subitem.name == 'chain.json'

      logger.info "Found directory #{item.name}"
      puts "Found directory #{item.name}"

      file_contents = client.contents(repo, path: subitem.path)
      text = Base64.decode64(file_contents.content)
      data = JSON.parse(text)

      chain_id = data['chain_id']

      if chain_id == nil || chain_id == '' || chain_id.empty?
        puts "Skipping #{item.name} because it's empty"
        next
      end

      logger.info "Reading #{chain_id}"
      puts "Reading #{chain_id}"

      reg[item.name] = chain_id
    end
  end
end

logger.info("Writing to file #{json_file}")
puts "Writing to file #{json_file}"
json_str = JSON.pretty_generate(reg)
File.write(json_file, json_str)
puts "Done!"
