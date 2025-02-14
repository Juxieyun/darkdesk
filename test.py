'''
Author: SpenserCai
Date: 2024-01-17 17:50:09
version: 
LastEditors: SpenserCai
LastEditTime: 2024-08-05 12:28:47
Description: file content
'''
import socket

def main():
    host = '127.0.0.1'  # Server IP address
    port = 19876  # Server port

    # Create a socket object
    client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    # Connect to the server
    client_socket.connect((host, port))
    print('Connected to {}:{}'.format(host, port))
    # message = '{"action":"create_new_connect","payload":{"id":"191305544","type":"connect","co_name":"聚水潭-山槐（马亮）","temporary_password":"2798gy","my_name":"聚水潭-山槐"}}'
    # message = '{"action":"create_new_connect","payload":{"id":"1989804443","type":"connect","co_name":"聚水潭-山槐（马亮）","temporary_password":"acmn36","my_name":"聚水潭-山槐"}}'
    # Send data to the server 1774637548
    # message = '{"action":"get_temporary_password", "payload":{"my_name":"聚水潭-山槐"}}'
    message = '{"action":"get_auto_recording"}'

#     message = """{
#     "action": "set_auto_recording",
#     "payload": {
#         "video_save_directory": "/Users/kerck/Downloads",
#         "allow_auto_record_incoming": "Y",
#         "allow_auto_record_outgoing": "Y"
#     }
# }"""

    # message = '{"co_name":"聚水潭-山槐（马亮）","id":"418588533","my_name":"聚水潭-山槐","temporary_password":"zerqnr","type":"connect"}'

    # message = '{"action":"create_new_connect", "payload":{"id":"122656574", "type":"connect", "co_name": "上海聚水潭RC专用挖掘机", "temporary_password":"frhbx3","my_name":"聚水潭-饕餮"}}'
    # message = '{"action":"create_new_connect", "payload":{"id":"122656574", "type":"file-transfer", "co_name": "被控公司名称", "temporary_password":"524567","my_name":"聚水潭-饕餮"}}'
    # message = '{"action":"get_server_status", "payload":{}}'
    # message = '{"action":"set_custom_server", "payload":{"id-server":"jxyr.juxieyun.com:21126", "relay-server":"jxyr.juxieyun.com:21147", "server-key":"al1+cElj3zk9TKIcrbiUGfojw3n0uGgLsbuCccO0wGI="}}'
    # local test
    # message = '{"action":"set_custom_server", "payload":{"id-server":"jxyr.juxieyun.com:21126", "relay-server":"127.0.0.1:21117", "server-key":"al1+cElj3zk9TKIcrbiUGfojw3n0uGgLsbuCccO0wGI="}}'
    # message = '{"action":"get_connection_status", "payload":{}}'
    # message = '{"action":"close_connection_by_id", "payload":{"id":"122656574","connect_type":"file-transfer"}}'
    
    client_socket.send(message.encode())

    # Receive the server's response
    response = client_socket.recv(1024).decode()
    print('Received from server:', response)

    # Close the connection
    client_socket.close()

if __name__ == '__main__':
    main()
