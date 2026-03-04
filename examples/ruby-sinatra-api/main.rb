require 'sinatra'
require 'json'

set :port, 4567

get '/' do
  content_type :json
  { message: 'Hello from Ruby + Sinatra!' }.to_json
end
