# frozen_string_literal: true

# read json file
require 'json'
require 'logger'
require 'fileutils'
require 'net/http'

working_dir = File.dirname(File.absolute_path(__FILE__))
Dir.chdir(working_dir)

chain_registry_file = 'chain_chainid.json'

# Logger

timestamp = Time.now.strftime('%Y%m%d%H%M%S')
log_name = "log-create_channels-#{timestamp}.log".to_s

logger = Logger.new(log_name)
logger.formatter = proc do |severity, datetime, _progname, msg|
  "[#{datetime.strftime('%Y-%m-%d %H:%M:%S')}] #{severity}: #{msg}\n"
end

logger.info("Reading #{chain_registry_file} file")
channels_before = JSON.parse(File.read(chain_registry_file))

logger.info('Updating chain registry')
worker = File.join(working_dir, 'update_chain_registry.rb')
system('ruby', worker)
channels_after = JSON.parse(File.read(chain_registry_file))

logger.info('Checking for new chains')
new_chains = channels_after.keys - channels_before.keys
logger.info("New chains: #{new_chains}")

logger.info('Checking for changed chain ids')
changed_chains = []
channels_before.each do |chain, chain_id|
  changed_chains << chain if chain_id != channels_after[chain]
end
logger.info("Changed chains: #{changed_chains}")

# Prep work folder

logger.info('Preparing work folder')
wdir = File.join(working_dir, 'channel_worker')
system("rm -rf #{wdir}") if Dir.exist?(wdir)

logger.info('Creating work folder')
Dir.mkdir(wdir)

logger.info('Copying files to work folder')
FileUtils.cp('./create-many-to-many.sh', wdir)

logger.info('Changing to work folder')
Dir.chdir(wdir) do
  logger.info('Fetching list of chains without connection')
  url = 'https://ibc.tfm.com/chain?status=Enabled&showChainsWithNoConnections=true'
  puts "Fetching #{url}"
  uri = URI(url)
  response = Net::HTTP.get(uri)
  channels = JSON.parse(response)

  logger.info('Writing target chain list to file')
  File.open('chains.txt', 'w') do |f|
    channels.each do |chain|
      f.puts(chain['chainName'])
    end
  end

  logger.info('Writing source chain list to file')
  File.open('source.txt', 'w') do |f|
    new_chains.each do |c|
      f.puts(c)
    end
  end

  if File.read('source.txt').empty?
    File.open('source.txt', 'w') do |f|
      f.puts('terra2')
    end
  end

  logger.info '---- Started channel creation ----'
  timestamp = Time.now.strftime('%Y%m%d%H%M%S')
  log_name = "log-#{timestamp}.log".to_s
  pid = spawn("./create-many-to-many.sh -r #{wdir}", out: log_name, err: log_name)
  Process.detach pid
  logger.info "Process #{pid} started"

  # system('./create-many-to-many.sh', '-r', wdir)
end
