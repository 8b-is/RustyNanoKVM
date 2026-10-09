import { useEffect } from 'react';
import { Button, notification } from 'antd';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';

import { isPasswordUpdated } from '@/api/auth.ts';
import { getSkipModifyPassword, setSkipModifyPassword } from '@/lib/localstorage.ts';

export const Notification = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [api, contextHolder] = notification.useNotification();

  useEffect(() => {
    const skip = getSkipModifyPassword();
    if (skip) return;

    let cancelled = false;
    isPasswordUpdated()
      .then((rsp) => {
        if (cancelled) return;
        if (rsp.code !== 0 || typeof rsp.data?.isUpdated !== 'boolean') {
          throw new Error('Password status unavailable');
        }
        if (!rsp.data.isUpdated) openNotification();
      })
      .catch(() => {
        if (cancelled) return;
        api.warning({
          key: 'password_status_unavailable',
          message: t('auth.passwordStatusUnavailable', {
            defaultValue: 'Could not check password status'
          }),
          description: t('auth.passwordStatusUnavailableDesc', {
            defaultValue: 'Your password status is unknown. Check your connection and try again later.'
          }),
          placement: 'topRight',
          duration: null
        });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  function openNotification() {
    api.warning({
      key: 'no_change_password',
      message: t('auth.changePassword'),
      description: t('auth.changePasswordDesc'),
      placement: 'topRight',
      btn: (
        <Button type="primary" onClick={changePassword}>
          {t('auth.ok')}
        </Button>
      ),
      duration: null,
      onClose: () => setSkipModifyPassword(true)
    });
  }

  function changePassword() {
    api.destroy();
    navigate('/auth/password');
  }

  return <>{contextHolder}</>;
};
