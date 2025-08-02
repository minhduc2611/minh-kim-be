import os
import smtplib
import ssl
import logging
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText
from typing import Optional

logger = logging.getLogger(__name__)

# Email configuration for PrivateEmail SMTP
SMTP_SERVER = os.getenv('SMTP_SERVER', 'mail.privateemail.com')
SMTP_PORT = int(os.getenv('SMTP_PORT', '587'))  # 587 for TLS, 465 for SSL
SMTP_USERNAME = os.getenv('SMTP_USERNAME')  # Your PrivateEmail address
SMTP_PASSWORD = os.getenv('SMTP_PASSWORD')  # Your PrivateEmail password
FROM_EMAIL = os.getenv('FROM_EMAIL', SMTP_USERNAME)
DOMAIN_URL = os.getenv('DOMAIN_URL', 'http://localhost:3000')

class EmailError(Exception):
    def __init__(self, message: str, status_code: int = 500):
        self.message = message
        self.status_code = status_code
        super().__init__(self.message)

def send_password_reset_email(email: str, reset_token: str) -> bool:
    """Send password reset email with magic link using PrivateEmail SMTP"""
    try:
        if not SMTP_USERNAME or not SMTP_PASSWORD:
            logger.error("SMTP credentials not configured")
            raise EmailError("Email service not configured", 500)
        
        # Create the reset link
        reset_link = f"{DOMAIN_URL}/reset-password?token={reset_token}"
        
        # Create message
        message = MIMEMultipart("alternative")
        message["Subject"] = "Password Reset Request"
        message["From"] = f"Buddha AI <{FROM_EMAIL}>"
        message["To"] = email
        
        # Email content
        html_content = f"""
        <html>
        <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
            <div style="text-align: center; margin-bottom: 30px;">
                <h1 style="color: #333;">Password Reset Request</h1>
            </div>
            
            <div style="background-color: #f9f9f9; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                <p style="color: #666; font-size: 16px; line-height: 1.5;">
                    We received a request to reset your password. If you didn't request this, please ignore this email.
                </p>
                
                <p style="color: #666; font-size: 16px; line-height: 1.5;">
                    To reset your password, click the button below:
                </p>
                
                <div style="text-align: center; margin: 30px 0;">
                    <a href="{reset_link}" 
                       style="background-color: #ca460b; color: white; text-decoration: none; 
                              padding: 12px 24px; border-radius: 6px; display: inline-block; 
                              font-weight: bold; font-size: 16px;">
                        Reset Password
                    </a>
                </div>
                
                <p style="color: #666; font-size: 14px; line-height: 1.5;">
                    If the button doesn't work, you can also copy and paste this link into your browser:
                </p>
                <p style="color: #ca460b; font-size: 14px; word-break: break-all;">
                    {reset_link}
                </p>
            </div>
            
            <div style="border-top: 1px solid #ddd; padding-top: 20px; color: #999; font-size: 12px;">
                <p>This link will expire in 1 hour for security reasons.</p>
                <p>If you have any questions, please contact our support team.</p>
            </div>
        </body>
        </html>
        """
        
        plain_content = f"""
        Password Reset Request
        
        We received a request to reset your password. If you didn't request this, please ignore this email.
        
        To reset your password, visit this link:
        {reset_link}
        
        This link will expire in 1 hour for security reasons.
        
        If you have any questions, please contact our support team.
        """
        
        # Create MimeText objects
        text_part = MIMEText(plain_content, "plain")
        html_part = MIMEText(html_content, "html")
        
        # Add parts to message
        message.attach(text_part)
        message.attach(html_part)
        
        # Send email using SMTP
        context = ssl.create_default_context()
        
        with smtplib.SMTP(SMTP_SERVER, SMTP_PORT) as server:
            server.starttls(context=context)  # Enable TLS encryption
            server.login(SMTP_USERNAME, SMTP_PASSWORD)
            server.sendmail(FROM_EMAIL, email, message.as_string())
        
        logger.info(f"Password reset email sent successfully to {email}")
        return True
        
    except smtplib.SMTPException as e:
        logger.error(f"SMTP error sending password reset email: {str(e)}")
        raise EmailError(f"Failed to send reset email: {str(e)}", 500)
    except Exception as e:
        logger.error(f"Error sending password reset email: {str(e)}")
        raise EmailError(f"Failed to send reset email: {str(e)}", 500)

def send_password_reset_confirmation_email(email: str) -> bool:
    """Send confirmation email after password reset using PrivateEmail SMTP"""
    try:
        if not SMTP_USERNAME or not SMTP_PASSWORD:
            logger.error("SMTP credentials not configured")
            return False
        
        # Create message
        message = MIMEMultipart("alternative")
        message["Subject"] = "Password Successfully Reset"
        message["From"] = f"Buddha AI <{FROM_EMAIL}>"
        message["To"] = email
        
        html_content = f"""
        <html>
        <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
            <div style="text-align: center; margin-bottom: 30px;">
                <h1 style="color: #ca460b;">Password Reset Successful</h1>
            </div>
            
            <div style="background-color: #f9f9f9; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                <p style="color: #666; font-size: 16px; line-height: 1.5;">
                    Your password has been successfully reset. You can now log in with your new password.
                </p>
                
                <p style="color: #666; font-size: 16px; line-height: 1.5;">
                    If you didn't make this change, please contact our support team immediately.
                </p>
                
                <div style="text-align: center; margin: 30px 0;">
                    <a href="{DOMAIN_URL}/login" 
                       style="background-color: #ca460b; color: white; text-decoration: none; 
                              padding: 12px 24px; border-radius: 6px; display: inline-block; 
                              font-weight: bold; font-size: 16px;">
                        Log In
                    </a>
                </div>
            </div>
            
            <div style="border-top: 1px solid #ddd; padding-top: 20px; color: #999; font-size: 12px;">
                <p>For security reasons, please ensure you're using a strong, unique password.</p>
            </div>
        </body>
        </html>
        """
        
        plain_content = f"""
        Password Reset Successful
        
        Your password has been successfully reset. You can now log in with your new password.
        
        If you didn't make this change, please contact our support team immediately.
        
        Visit {DOMAIN_URL}/login to log in.
        
        For security reasons, please ensure you're using a strong, unique password.
        """
        
        # Create MimeText objects
        text_part = MIMEText(plain_content, "plain")
        html_part = MIMEText(html_content, "html")
        
        # Add parts to message
        message.attach(text_part)
        message.attach(html_part)
        
        # Send email using SMTP
        context = ssl.create_default_context()
        
        with smtplib.SMTP(SMTP_SERVER, SMTP_PORT) as server:
            server.starttls(context=context)  # Enable TLS encryption
            server.login(SMTP_USERNAME, SMTP_PASSWORD)
            server.sendmail(FROM_EMAIL, email, message.as_string())
        
        return True
        
    except Exception as e:
        logger.error(f"Error sending password reset confirmation email: {str(e)}")
        return False 