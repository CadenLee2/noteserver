# Administration for https://github.com/Cadecraft/noteserver

import requests
import os

LOCAL_URL = 'http://localhost:3002'
PROD_URL = 'https://notes.cadenlee.dev'

# Helpers
def help():
    print('Commands:')
    print('- q                                       | Exit')
    print('- help                                    | Print commands list')
    print('- ls                                      | List all directories and tokens')
    print('File management:')
    print('- post {dir_id}/{note_id} {localfile.md}  | Add note from local file')
    print('- post-bulk {dir_id} {localfoldername}    | Post all markdown files inside a given folder')
    print('- post-dir {dir_id}                       | Update directory info')
    print('- delete! {dir_id}/{note_id}              | Delete a note')
    print('- delete-dir! {dir_id}                    | Delete a directory')
    print('Token management:')
    print('- post-tok {dir_id}                       | Create a token for a directory')
    print('- delete-tok! {tok}                       | Delete a token')
    print();

def read_file(path):
    file = open(path, "r", encoding='utf-8')
    content = file.read()
    file.close()
    return content

def post_bulk(remote_dir, local_folder):
    to_post = []
    for entry in os.listdir(local_folder):
        full_path = os.path.join(local_folder, entry)
        if os.path.isdir(full_path):
            continue
        if not entry.endswith('.md'):
            continue
        simplified = str(entry)[0:(len(entry)-3)]
        to_post.append((simplified, full_path))
        print('Found ' + full_path)

    print('---')

    if input('Confirm: post ' + str(len(to_post)) + ' files? y/N: ') != 'y':
        print('Cancelled')
        return
    for entry in to_post:
        md_content = read_file(entry[1])
        print('Posting ' + entry[1])
        resp = requests.post(api_url + '/' + remote_dir + '/' + entry[0], md_content, headers=auth_headers)
        print('Server returned ' + str(resp.status_code))

# Start input
print('noteserver admin')
print()

print("Shortcuts: 'l' for " + LOCAL_URL + ", 'p' for " + PROD_URL)
api_url = input("Connection url: ")
if api_url == 'l':
    api_url = LOCAL_URL
elif api_url == 'p':
    api_url = PROD_URL
password = input('Enter your admin password: ')
print()

help()

while True:
    user_command = input('> ')
    args = user_command.split(' ')

    auth_headers = {'Authorization': password}

    try:
        if (args[0] == 'q' or args[0] == 'exit'):
            break
        elif (args[0] == 'help'):
            help()
        elif (args[0] == 'ls'):
            resp = requests.get(api_url + '/all', headers=auth_headers)
            print(resp.text)
        elif (args[0] == 'post'):
            remote_note = args[1]
            local_file = args[2]
            md_contents = read_file(local_file)
            resp = requests.post(api_url + '/' + remote_note, md_contents, headers=auth_headers)
            print(resp.status_code)
        elif (args[0] == 'post-dir'):
            remote_dir = args[1]
            protected = input('Protected? y/N: ') == 'y'
            descr = input('Description: ')
            resp = requests.post(api_url + '/' + remote_dir + '?protected=' + ('true' if protected else 'false'), descr, headers=auth_headers)
            print(resp.status_code)
        elif (args[0] == 'post-bulk'):
            remote_dir = args[1]
            local_folder = args[2]
            post_bulk(remote_dir, local_folder)
        elif (args[0] == 'delete!'):
            remote_note = args[1]
            resp = requests.delete(api_url + '/' + remote_note, headers=auth_headers)
            print(resp.status_code)
        elif (args[0] == 'delete-dir!'):
            remote_dir = args[1]
            resp = requests.delete(api_url + '/' + remote_dir, headers=auth_headers)
            print(resp.status_code)
        elif (args[0] == 'post-tok'):
            remote_dir = args[1]
            tok = input('Enter token: ')
            resp = requests.post(api_url + '/token/' + tok + '?directory=' + remote_dir, headers=auth_headers)
            print(resp.status_code)
        elif (args[0] == 'delete-tok!'):
            tok = args[1]
            resp = requests.delete(api_url + '/token/' + tok, headers=auth_headers)
            print(resp.status_code)
        else:
            print('Invalid command')
    except Exception as e:
        print('Error running command: ' + str(e))

    print(' ')
