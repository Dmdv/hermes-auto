# Specs

## Quote from the messenger:

> 1 - A complete workflow from A-Z for two new chains which are missing connections. (like a use case for using the scripts + commands)  
> 2 - A detailed description of each script (input parameters, description, expected results, dependencies with other scripts)   
> 3 - A final report covering the development status of each script (compatibility, what is left etc.)  
> 4 - Documentation in detail, not just bullet points and screenshots without description -  
> Proper document which describes the workflow of setting up the working repository from scratch  

> * Which libraries, dependencies to install (including install/build commands)
> * If anything needs to manually set up or needs to be altered (for example chain files, folder structure, other manual tasks?)
> * Case Study setting up connections for 1 chain (eg phoenix-1) A-Z -> all steps + all scripts step by step + potential errors and how to deal with them, every step until we have the config files
> * Steps to start hermes after we created config files
> * Detailed description of every "run" mode implemented (multiple instances, single instance)
> * (Should be already in the script description but extensive details how the health check service works + rpc/rest/grpc getting (the service built to handle rate limiting))

## DISCLAIMER

1. I am not duplicating documentation of Hermes which is already available at https://hermes.informal.systems/index.html 
2. An engineer who will be working with Hermes should be familiar with the documentation above and how hermes works. It requires only basic knowledge of scripting languages to understand the logic in scripts - that was the purpose of scripts.
3. Some of the points are repeating itself.
4. These scripts were used for the current hermes state as it is developed on the day when I finished working for TFM.
5. ALL these scripts were developed according to the discussion over daily calls.
6. I do not take responsibility for any changes that might have been made to the Hermes code after I left.
7. I do not take responsibility for misconfiguration of server setup. For example, I advised to have multiple services 
to be deployed on multiple servers in docker instances which would be run automatically by docker-compose or any other docker automation tool.
8. I am not responsible for proxy server development to avoid issue with rate limiting. 
   - Rate limiting is not the part of a relayer.
   - I advised to use a forward proxy server to avoid rate limiting issues.
   - I advised to use proxychains to avoid rate limiting issues but this is not a final high-end solution.
   - Rate limiting is a feature of the common cosmos-based blockchain node, and it can be avoided only by using a proxy server or placing hermes instances on different servers, because any relayer instance will spam nodes.

## Answer to the quote

The documentation presented here describe how to install, initialize, and run Hermes relayer.

1. NEW_CHAINS.MD
2. SCRIPTS.MD
3. All scripted were used for initialization, channels creation and instance management. Nothing is left to be done manually.
4. Yes, all documentation is in detail. 