#!/usr/bin/env ruby
# frozen_string_literal: true

require 'English'
require 'gli'
require 'net/http'
require 'json'
require 'colorize'
require 'colorized_string'
require 'logger'
require 'os'

include GLI::App

working_dir = File.dirname(File.absolute_path(__FILE__))
Dir.chdir(working_dir)

timestamp = Time.now.strftime('%Y%m%d%H%M%S')
log_name = "log-hermes-many-#{timestamp}.log".to_s

# Create a logger for the file
# file_logger = Logger.new(log_name)

# Create a logger for the console
# console_logger = Logger.new(STDOUT)

# Create a combined logger that logs to both the file and the console
# logger = Logger.new(STDOUT)
# logger.formatter = proc { |severity, datetime, _progname, msg| "[#{datetime.strftime('%Y-%m-%d %H:%M:%S')}] #{severity}: #{msg}\n" }
# logger << file_logger
# logger << console_logger

logger = Logger.new(log_name)
logger.formatter = proc do |severity, datetime, _progname, msg|
  "[#{datetime.strftime('%Y-%m-%d %H:%M:%S')}] #{severity}: #{msg}\n"
end

def proxychains_name
  if OS.linux?
    'proxychains'
  elsif OS.mac?
    'proxychains4'
  end
end

def process_file_name
  'process.pid'
end

def clean
  File.delete(process_file_name) if File.exist?(process_file_name)
end

def hermes_started
  File.exist?(process_file_name) && !File.read(process_file_name).empty?
end

def hermes_pid
  File.read(process_file_name).to_i
end

def hermes_running
  hermes_started && system("ps -p #{hermes_pid} >/dev/null")
  # if File.exist?(pid_file)
  #   pid = File.read(pid_file).to_i
  #   if pid > 0 && Process.kill(0, pid) == 1
  #     puts "Process with PID #{pid} is still running"
  #   else
  #     puts "Process with PID #{pid} is not running"
  #   end
  # else
  #   puts "PID file #{pid_file} not found"
  # end
end

def hermes_stopped
  !hermes_running
end

# Command updates the proxy configuration
desc 'Check if proxy installed'
command('proxy-update') do |c|
  c.action do |global_options, _, _|
    home = global_options[:home]
    info "Using home folder: #{home}"

    # check is proxy installed

    # proxychains = "proxychains4 -f #{home}/proxychains.conf"
    unless is_proxy_installed
      logger.warn "#{proxychains_name} is not installed. Exiting..."
      exit 1
    end

    # prepare config file

    cfg_path = File.join(ENV['HOME'], '.proxychains', 'proxychains.conf')
    File.delete(cfg_path) if File.exist?(cfg_path)

    # Get proxy list from
    # https://www.proxy-list.download/api/v1/get?type=http&anon=elite&country=US

    url = 'https://proxy.webshare.io/api/v2/proxy/list/?mode=direct&page_size=250'
    uri = URI(url.to_s)

    http = Net::HTTP.new(uri.host, uri.port)
    http.use_ssl = true

    info "Fetching #{url}"

    request = Net::HTTP::Get.new(url)
    request['Authorization'] = 'e4pdtrgtqqdn8fc9zxlmy7k8ptoe8icf88g0s7xl'
    response = http.request(request)
    json_response = JSON.parse(response.body)

    # create config file

    results = json_response['results']

    File.open(cfg_path, 'a') do |f|
      f.puts 'random_chain'
      f.puts 'chain_len = 1'
      f.puts 'proxy_dns'
      f.puts 'remote_dns_subnet 224'
      f.puts 'tcp_read_time_out 15000'
      f.puts 'tcp_connect_time_out 8000'
      f.puts 'localnet 127.0.0.0/255.0.0.0'
      f.puts '[ProxyList]'

      results.each do |result|
        f.puts "http\t#{result['proxy_address']}\t#{result['port']}\t#{result['username']}\t#{result['password']}"
      end
    end
  end
end

def is_proxy_installed
  if system("which #{proxychains_name}")
    puts "#{proxychains_name} is installed"
    true
  else
    puts "#{proxychains_name} is not installed"
    false
  end
end

def start_hermes(config_path, use_proxy)
  info "Starting hermes with config #{config_path}"
  unless File.exist?('endpoints_status.log') && File.read('endpoints_status.log') == 'updated'
    # need to compare with current time
    info 'Updating endpoints...'
    pid = spawn("hermes --config #{config_path} config endpoints", out: 'endpoints.log', err: 'endpoints.log')
    Process.wait(pid)
    info 'Endpoints updated SUCCESSFULLY'
    File.write('endpoints_status.log', 'updated')
    # system("hermes --config #{config_path} config_path endpoints > 'endpoints.txt' 2>&1 &")
    # system("hermes --config #{config_path} config_path endpoints")
    # pid = $CHILD_STATUS.pid
    # Process.wait(pid)
  end

  timestamp = Time.now.strftime('%Y%m%d%H%M%S')
  log_name = "log-#{timestamp}.log".to_s

  # system("hermes --config #{config_path} start > #{log_name} 2>&1 &")
  # pid = $?.pid

  pid = if use_proxy
          spawn("#{proxychains_name} hermes --config #{config_path} start", out: log_name, err: log_name)
        else
          spawn("hermes --config #{config_path} start", out: log_name, err: log_name)
        end

  info "Starting hermes with PID #{pid}"
  Process.detach pid
  File.write(process_file_name, pid)
end

def stop_hermes
  return unless hermes_running

  pid = hermes_pid
  info "Stopping #{pid}"
  Process.kill('TERM', pid)
  File.delete(process_file_name)
end

def warn(msg)
  puts 'WARN: '.yellow + msg
end

def err(msg)
  puts 'ERROR: '.red + msg
end

def info(msg)
  puts 'INFO: '.green + msg
end

program_desc 'Configure and run hermes instances using channels registry'

desc 'Init single hermes instance with one config file for one chain. Relaying 1 -> N channels'
command 'init-one' do |c|
  c.desc 'Delete existing folder'
  c.switch %i[f force]

  c.desc 'Chain for which need to create folder'
  c.arg_name 'CHAIN'
  c.flag %i[c chain]

  c.action do |global_options, options, args|
    # force_delete = options[:force]
    chain = options[:chain] || args.first

    if chain.nil? || chain.empty? || chain == ' '
      logger.info 'Chain not provided. Exiting...'
      exit 1
    end

    chain.downcase!
    home = global_options[:home]
    logger.info "Using home folder: #{home}"

    # Default file is causing problems when hermes is started
    default_cfg = File.join(home, 'config.toml')
    File.delete(default_cfg) if File.exist?(default_cfg)

    unless File.exist?('prepare-config.sh')
      logger.info 'prepare-config.sh not found. Exiting...'
      exit 1
    end

    unless File.exist?('chain_chainid.json')
      logger.info 'chain_chainid.json not found. Exiting...'
      exit 1
    end

    chains_reg = JSON.parse(File.read('chain_chainid.json'))
    unless chains_reg.key?(chain)
      logger.info "Chain #{chain} not found in chain_chainid.json. Exiting..."
      exit 1
    end

    source_chain_id = chains_reg[chain]
    chain_id_reg = chains_reg.invert

    channel_reg_file = File.join(home, "#{source_chain_id}-channels.json")

    unless File.exist?(channel_reg_file)
      logger.info "Chain registry #{channel_reg_file} not found. Exiting..."
    end

    if File.empty?(channel_reg_file)
      logger.info "Chain registry #{channel_reg_file} is empty. Exiting..."
    end

    last_rest_port = 3000
    last_tm_port = last_rest_port + 1

    file_name = File.basename(channel_reg_file)

    index = file_name.index('-channels.json')
    chain_id = ''
    chain_id = file_name.slice(0, index) if index
    if chain_id.nil? || chain_id.empty?
      logger.info "Invalid file name #{file_name}"
      exit 1
    end

    logger.info "Processing channels from #{chain_id} in registry #{file_name}"

    data = JSON.parse(File.read(channel_reg_file))
    source_chain_id = data['chain']['chain_id']
    channels = data['channels']

    unless chain_id_reg.key?(source_chain_id)
      logger.info "Chain id #{source_chain_id}not found in registry. Exiting..."
      exit 1
    end

    hermes_dir = "#{home}/#{chain}_hermes"
    if Dir.exist?(hermes_dir)
      logger.info "Folder #{hermes_dir} exists. Deleting..."
      system("rm -rf #{hermes_dir}")
    end

    Dir.mkdir(hermes_dir)
    dest_file = "#{hermes_dir}/dest.txt"
    File.open(dest_file, 'a') do |f|

      f.puts chain_id_reg[source_chain_id]

      channels.each do |channel|
        target_chain_id = channel['chain']['chain_id']
        logger.info "Adding chain to config #{source_chain_id} => #{target_chain_id}"

        unless chain_id_reg.key?(target_chain_id)
          logger.warn "Chain id #{target_chain_id} not found in registry. Skipping..."
          next
        end

        f.puts chain_id_reg[target_chain_id]

      end
    end

    source_chain = chain_id_reg[source_chain_id].to_s

    logger.info "---- Starting autoconfiguration with source #{source_chain} ----"

    # timestamp = Time.now.strftime('%Y%m%d%H%M%S')
    # log_name = "log-#{timestamp}.log".to_s
    # log_file = File.join(hermes_dir, log_name)

    # pid = spawn("./prepare-config.sh -f #{dest_file} -s #{source_chain} -r #{hermes_dir} -p #{last_rest_port} -m #{last_tm_port}",
    #       out: log_file,
    #       err: log_file)
    #
    # Process.wait(pid)
    #
    system('./prepare-config.sh',
           '-f', dest_file,
           '-s', source_chain,
           '-r', hermes_dir,
           '-p', last_rest_port.to_s,
           '-m', last_tm_port.to_s)

    logger.info '---- Completed autoconfiguration ----'

  end
end

desc 'Start single hermes instance with one config file for one chain. Relaying 1 -> N channels'
command 'start-one' do |c|
  c.desc 'Chain for which need to start instance'
  c.arg_name 'CHAIN'
  c.flag %i[c chain]

  c.desc 'Run with proxy'
  c.switch %i[p proxy]

  c.action do |global_options, options, args|
    # arguments
    chain = options[:chain] || args.first
    use_proxy = options[:proxy]
    if use_proxy
      info 'Using proxy'
    else
      info 'Not using proxy'
    end

    if chain.nil? || chain.empty? || chain == ' '
      logger.info 'Chain not provided. Exiting...'
      exit 1
    end

    chain.downcase!
    home = global_options[:home]
    logger.info "Using home folder: #{home}"

    # Default file is causing problems when hermes is started
    default_cfg = File.join(home, 'config.toml')
    File.delete(default_cfg) if File.exist?(default_cfg)

    unless File.exist?('chain_chainid.json')
      logger.info 'chain_chainid.json not found. Exiting...'
      exit 1
    end

    chains_reg = JSON.parse(File.read('chain_chainid.json'))
    unless chains_reg.key?(chain)
      logger.info "Chain #{chain} not found in chain_chainid.json. Exiting..."
      exit 1
    end

    dir = "#{home}/#{chain}_hermes"
    config_path = File.join(dir, 'config.toml')

    Dir.chdir(dir) do
      if hermes_running
        warn 'Hermes is already running. Skipping...'
      else
        clean
        start_hermes(config_path, use_proxy)
      end
    end

  end
end

desc 'Stop single hermes instance with one config file for one chain. Relaying 1 -> N channels'
command 'stop-one' do |c|
  c.desc 'Chain for which need to start instance'
  c.arg_name 'CHAIN'
  c.flag %i[c chain]

  c.action do |global_options, options, args|
    # arguments
    chain = options[:chain] || args.first

    if chain.nil? || chain.empty? || chain == ' '
      logger.info 'Chain not provided. Exiting...'
      exit 1
    end

    chain.downcase!
    home = global_options[:home]
    logger.info "Using home folder: #{home}"

    unless File.exist?('chain_chainid.json')
      logger.info 'chain_chainid.json not found. Exiting...'
      exit 1
    end

    chains_reg = JSON.parse(File.read('chain_chainid.json'))
    unless chains_reg.key?(chain)
      logger.info "Chain #{chain} not found in chain_chainid.json. Exiting..."
      exit 1
    end

    dir = "#{home}/#{chain}_hermes"

    warn "Folder #{dir} not found. Exiting..." unless Dir.exist?(dir)

    Dir.chdir(dir) do
      if hermes_running
        info 'Hermes is running. Stopping...'
        stop_hermes
        clean
      end
    end
  end
end

desc 'Init multi folder hermes instance using channels registry json files'
command :init do |c|
  c.desc 'Delete existing folder'
  c.switch %i[f force]

  c.action do |global_options, options, _|
    force_delete = options[:force]
    home = global_options[:home]
    info "Using HOME: #{home}"

    unless File.exist?('prepare-config.sh')
      puts 'prepare-config.sh not found. Exiting'
      exit 1
    end

    unless File.exist?('chain_chainid.json')
      puts 'chain_chainid.json not found. Exiting'
      exit 1
    end

    chains_reg = JSON.parse(File.read('chain_chainid.json'))
    chain_id_reg = chains_reg.invert

    last_rest_port = 3000
    last_tm_port = last_rest_port + 1

    # Loop over all files matching the pattern xxx-channels.json
    Dir.glob("#{home}/*-channels.json").each do |file|
      next if File.directory?(file)

      if File.empty?(file)
        puts "Empty #{file}. Skipping"
        next
      end

      file_name = File.basename(file)

      index = file_name.index('-channels.json')
      chain_id = ''
      chain_id = file_name.slice(0, index) if index
      if chain_id.nil? || chain_id.empty?
        puts "Invalid file name #{file_name}"
        next
      end

      puts "Processing channels from #{chain_id} in registry #{file_name}"

      data = JSON.parse(File.read(file))
      source_chain_id = data['chain']['chain_id']
      channels = data['channels']

      channels.each do |channel|
        # probing available ports
        rest_port = last_rest_port
        tm_port = last_tm_port

        loop do
          if !system("nc -z localhost #{last_rest_port} >/dev/null")
            rest_port = last_rest_port
            last_rest_port += 1
            break
          else
            last_rest_port += 1
          end
        end

        last_tm_port = last_rest_port

        loop do
          if !system("nc -z localhost #{last_tm_port} >/dev/null")
            tm_port = last_tm_port
            last_tm_port += 1
            break
          else
            last_tm_port += 1
          end
        end

        last_rest_port = last_tm_port

        target_chain_id = channel['chain']['chain_id']
        puts '******************************************************************'
        puts "Adding config for #{source_chain_id} => #{target_chain_id}"
        puts '******************************************************************'

        puts "Using #{rest_port} for REST"
        puts "Using #{tm_port} for TELEMETRY"

        dir = "#{home}/#{source_chain_id}_#{target_chain_id}"
        system("rm -rf #{dir}") if Dir.exist?(dir) && force_delete
        Dir.mkdir(dir) unless Dir.exist?(dir) && force_delete

        puts "Created folder #{dir}"
        dest_file = "#{dir}/dest.txt"

        File.delete(dest_file) if File.exist?(dest_file)
        File.open(dest_file, 'a') do |f|
          if !chain_id_reg.key?(source_chain_id) || !chain_id_reg.key?(target_chain_id)
            puts "Chain id #{source_chain_id} or #{target_chain_id} not found in registry. Skipping..."
            next
          end
          f.puts chain_id_reg[source_chain_id]
          f.puts chain_id_reg[target_chain_id]
        end

        source_chain = chain_id_reg[source_chain_id].to_s

        info "---- Starting autoconfiguration with source #{source_chain} ----"

        system('./prepare-config.sh',
               '-f', dest_file,
               '-s', source_chain,
               '-r', dir,
               '-p', rest_port.to_s,
               '-m', tm_port.to_s)

        info '---- Completed autoconfiguration ----'
      end
    end
  end
end

desc 'Show chain config from registry'
command :config do |c|
  c.desc 'The chain to show'
  c.arg_name 'CHAIN'
  c.flag %i[c chain]

  c.action do |_, options, args|
    chain = options[:chain] || args.first
    chain.downcase!

    puts '----------------------'
    puts " #{chain}"
    puts '----------------------'

    url = "https://raw.githubusercontent.com/cosmos/chain-registry/master/#{chain}/chain.json"
    puts "Fetching #{url}"
    uri = URI(url)
    response = Net::HTTP.get(uri)

    data = JSON.parse(response)
    puts JSON.pretty_generate(data)
  end
end

desc 'Show running instances'
command :show do |c|
  c.action do |global_options, _, _|
    home = global_options[:home]
    info "Using home folder: #{home}"

    counter = 0

    Dir.glob("#{home}/*").each do |dir|
      next unless File.directory?(dir)

      dir_name = File.basename(dir)
      chains = dir_name.split('_')
      config_path = File.join(dir, 'config.toml')
      next if chains.length != 2 || !File.exist?(config_path)

      Dir.chdir(dir) do
        if hermes_running
          puts "PID: #{hermes_pid} (#{chains[0]} => #{chains[1]})"
          counter += 1
        end
      end
    end

    puts '-' * 40
    info "Total #{counter} running instances"

  end
end

desc 'Stop running instances'
command :stop do |c|
  c.action do |global_options, _, _|
    home = global_options[:home]
    info "Using home folder: #{home}"

    Dir.glob("#{home}/*").each do |dir|
      next unless File.directory?(dir)

      dir_name = File.basename(dir)
      chains = dir_name.split('_')
      config_path = File.join(dir, 'config.toml')
      next if chains.length != 2 || !File.exist?(config_path)

      Dir.chdir(dir) do
        if hermes_running
          puts "PID: #{hermes_pid} (#{chains[0]} => #{chains[1]})"
          stop_hermes
        end
      end
    end
  end
end

desc 'Start hermes instances'
command :start do |c|
  c.desc 'Source chain'
  c.arg_name 'SRC'
  c.flag %i[s src]

  c.desc 'Destination chain'
  c.arg_name 'DST'
  c.flag %i[d dst]

  c.desc 'Run all instances'
  c.switch %i[a all]

  c.desc 'Run with proxy'
  c.switch %i[p proxy]

  c.action do |global_options, options, _|
    home = global_options[:home]
    info "Using home folder: #{home}"

    run_all = options[:all]
    use_proxy = options[:proxy]
    if use_proxy
      info 'Using proxy'
    else
      info 'Not using proxy'
    end

    if run_all
      info 'All chains will be started'
      info 'Source and destination chain will be ignored'

      Dir.glob("#{home}/*").each do |dir|
        next unless File.directory?(dir)

        dir_name = File.basename(dir)
        chains = dir_name.split('_')
        config_path = File.join(dir, 'config.toml')
        next if chains.length != 2 || !File.exist?(config_path)

        info "Trying #{dir_name}".light_yellow
        info "Config file: #{config_path}"

        Dir.chdir(dir) do
          if hermes_running
            warn 'Hermes is already running. Skipping...'
          else
            clean
            start_hermes(config_path, use_proxy)
          end
        end
      end

    else
      src = options[:src]
      info "Source chain specified: #{src}" unless src.nil? || src.empty?

      dst = options[:dst]
      info "Destination chain specified: #{dst}" unless dst.nil? || dst.empty?
    end

  end
end

desc 'Show channels used by chain'
command :channels do |c|
  c.desc 'The source chain'
  c.arg_name 'CHAIN'
  c.flag %i[c chain]

  c.action do |global_options, options, args|
    chain = options[:chain] || args.first
    chain.downcase!
    home = global_options[:home]
    info "Using home folder: #{home}"

    name = File.join(home, "#{chain}-channels.json")

    if !File.exist?(name) || File.empty?(name)
      warn "File #{name} not found... Looking in registry"
      # perhaps it it the chain name but not chain id
      url = "https://raw.githubusercontent.com/cosmos/chain-registry/master/#{chain}/chain.json"
      info "Fetching #{url}"
      uri = URI(url)
      response = Net::HTTP.get(uri)
      data = JSON.parse(response)
      chain_id = data['chain_id']

      name = File.join(home, "#{chain_id}-channels.json")
      unless File.exist?(name)
        err "File #{name} not found... Exiting"
        exit 1
      end
    end

    info "Using registry #{name}"
    data = JSON.parse(File.read(name))
    source_chain_id = data['chain']['chain_id']
    channels = data['channels']
    info "Source chain: #{source_chain_id}".colorize(color: :light_blue, mode: :bold)

    sorted = channels.sort { |a, b| a['chain']['chain_id'] <=> b['chain']['chain_id'] }
    sorted.each do |channel|
      target_chain_id = channel['chain']['chain_id']
      info target_chain_id.colorize(color: :grey)
    end
  end
end

# Define a global option
desc 'The output format'
arg_name 'HOME'
default_value File.join(ENV['HOME'], '.hermes')
flag %i[h home]

on_error do |exception|
  puts exception.message
  # return false to skip default error handling
  exit 1
end

# # Define a default command
# default_command :hello

# Run the command-line tool
exit run(ARGV)
